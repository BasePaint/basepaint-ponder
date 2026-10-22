use crate::calls::CallReq;
use crate::logs;
use anyhow::{Result, anyhow, bail};
use futures::stream::{self, StreamExt};
use serde_json::{Value, json};
use std::time::Duration;

/// JSON-RPC client that tries each URL in order, like viem's `fallback` transport.
#[derive(Clone)]
pub struct Rpc {
    urls: Vec<String>,
    http: reqwest::Client,
}

pub enum RpcError {
    /// The node answered with a JSON-RPC error (deterministic, e.g. a revert or a range limit).
    Node(String),
    Transport(anyhow::Error),
}

impl Rpc {
    pub fn new(urls: Vec<String>, timeout: Duration) -> Rpc {
        Rpc {
            urls,
            http: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .expect("http client"),
        }
    }

    async fn call_once(&self, url: &str, method: &str, params: &Value) -> Result<Value, RpcError> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        let res = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Transport(e.into()))?;
        let status = res.status();
        let j: Value = res
            .json()
            .await
            .map_err(|e| RpcError::Transport(anyhow!("{status}: {e}")))?;
        if let Some(err) = j.get("error") {
            let msg = err["message"].as_str().unwrap_or_default().to_string();
            let code = err["code"].as_i64().unwrap_or_default();
            // Rate limits and capacity errors are worth retrying elsewhere.
            if status.as_u16() == 429
                || code == 429
                || msg.contains("rate")
                || msg.contains("capacity")
                || msg.contains("exceeded your")
            {
                return Err(RpcError::Transport(anyhow!("{msg}")));
            }
            return Err(RpcError::Node(format!("{code}: {msg}")));
        }
        Ok(j["result"].clone())
    }

    /// Tries every URL, retrying transport errors with backoff. Node errors return immediately.
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let mut last = None;
        for attempt in 0..5u32 {
            for url in &self.urls {
                match self.call_once(url, method, &params).await {
                    Ok(v) => return Ok(v),
                    Err(RpcError::Node(m)) => return Err(RpcError::Node(m)),
                    Err(RpcError::Transport(e)) => last = Some(e),
                }
            }
            tokio::time::sleep(Duration::from_millis(250 << attempt)).await;
        }
        Err(RpcError::Transport(
            last.unwrap_or_else(|| anyhow!("no RPC URLs configured")),
        ))
    }

    pub async fn request_ok(&self, method: &str, params: Value) -> Result<Value> {
        match self.request(method, params).await {
            Ok(v) => Ok(v),
            Err(RpcError::Node(m)) => bail!("{method}: {m}"),
            Err(RpcError::Transport(e)) => Err(e.context(method.to_string())),
        }
    }

    pub async fn block_number(&self) -> Result<u64> {
        let v = self.request_ok("eth_blockNumber", json!([])).await?;
        Ok(u64::from_str_radix(
            v.as_str().unwrap_or("0x0").trim_start_matches("0x"),
            16,
        )?)
    }

    /// All BasePaint logs in [from, to], as cache records sorted by (block, logIndex).
    /// Splits the range whenever the node refuses it as too large.
    pub async fn get_logs(&self, from: u64, to: u64) -> Result<Vec<u8>> {
        const STEP: u64 = 100_000;
        let ranges: Vec<(u64, u64)> = (from..=to)
            .step_by(STEP as usize)
            .map(|b| (b, (b + STEP - 1).min(to)))
            .collect();
        let chunks: Vec<Result<Vec<u8>>> = stream::iter(ranges)
            .map(|(a, b)| self.get_logs_range(a, b))
            .buffered(8)
            .collect()
            .await;
        let mut out = Vec::new();
        for c in chunks {
            out.extend_from_slice(&c?);
        }
        Ok(out)
    }

    fn get_logs_range(
        &self,
        from: u64,
        to: u64,
    ) -> futures::future::BoxFuture<'_, Result<Vec<u8>>> {
        Box::pin(async move {
            let params = json!([{ "address": logs::CONTRACTS, "fromBlock": format!("0x{from:x}"), "toBlock": format!("0x{to:x}") }]);
            match self.request("eth_getLogs", params).await {
                Ok(v) => {
                    let mut buf = Vec::new();
                    for l in v
                        .as_array()
                        .ok_or_else(|| anyhow!("eth_getLogs: expected array"))?
                    {
                        if l["removed"].as_bool() == Some(true) {
                            continue;
                        }
                        logs::encode_rpc_log(l, &mut buf)?;
                    }
                    Ok(logs::sort_records(&buf))
                }
                Err(RpcError::Node(m)) if from < to => {
                    let mid = from + (to - from) / 2;
                    let mut a = self.get_logs_range(from, mid).await?;
                    a.extend_from_slice(
                        &self
                            .get_logs_range(mid + 1, to)
                            .await
                            .map_err(|e| e.context(m))?,
                    );
                    Ok(a)
                }
                Err(RpcError::Node(m)) => bail!("eth_getLogs {from}: {m}"),
                Err(RpcError::Transport(e)) => Err(e.context("eth_getLogs")),
            }
        })
    }

    /// Runs eth_calls; a revert is a valid (cached) result, transport errors fail the batch.
    pub async fn eth_calls(&self, reqs: Vec<CallReq>) -> Result<Vec<(CallReq, Option<Vec<u8>>)>> {
        let results: Vec<Result<(CallReq, Option<Vec<u8>>)>> = stream::iter(reqs)
            .map(|req| async move {
                let params =
                    json!([{ "to": req.to, "data": req.data }, format!("0x{:x}", req.block)]);
                match self.request("eth_call", params).await {
                    Ok(v) => {
                        let bytes =
                            hex::decode(v.as_str().unwrap_or("0x").trim_start_matches("0x"))?;
                        Ok((req, Some(bytes)))
                    }
                    Err(RpcError::Node(m)) if m.contains("revert") || m.starts_with("3:") => {
                        Ok((req, None))
                    }
                    Err(RpcError::Node(m)) => bail!("eth_call {}@{}: {m}", req.data, req.block),
                    Err(RpcError::Transport(e)) => Err(e.context("eth_call")),
                }
            })
            .buffer_unordered(16)
            .collect()
            .await;
        results.into_iter().collect()
    }
}
