// Cache of eth_call results the handlers depend on (Chainlink price, brush strengths,
// epoch duration). Like Ponder's RPC cache: fetched once, persisted, replayed on reindex.

use crate::state::keccak;
use rustc_hash::FxHashMap;
use std::io::Write;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallReq {
    pub to: String,
    pub data: String,
    pub block: u64,
}

impl CallReq {
    pub fn new(to: &str, sig: &str, arg: Option<u64>, block: u64) -> CallReq {
        let mut data = hex::encode(&keccak(sig.as_bytes())[..4]);
        if let Some(a) = arg {
            data.push_str(&format!("{:064x}", a));
        }
        CallReq {
            to: to.to_lowercase(),
            data: format!("0x{data}"),
            block,
        }
    }

    pub fn eth_usd_price(feed: &str, block: u64) -> CallReq {
        CallReq::new(feed, "latestRoundData()", None, block)
    }

    fn key(&self) -> String {
        format!("{}:{}:{}", self.to, self.data, self.block)
    }
}

/// Result bytes, or None when the call reverted.
#[derive(Default)]
pub struct CallCache {
    map: FxHashMap<String, Option<Vec<u8>>>,
    path: Option<std::path::PathBuf>,
}

impl CallCache {
    pub fn load(path: &std::path::Path) -> anyhow::Result<CallCache> {
        let mut map = FxHashMap::default();
        if let Ok(s) = std::fs::read_to_string(path) {
            for line in s.lines() {
                if let Some((k, v)) = line.split_once('\t') {
                    map.insert(
                        k.to_string(),
                        if v == "REVERT" {
                            None
                        } else {
                            Some(hex::decode(v)?)
                        },
                    );
                }
            }
        }
        Ok(CallCache {
            map,
            path: Some(path.to_path_buf()),
        })
    }

    pub fn missing(&self, reqs: &[CallReq]) -> Vec<CallReq> {
        let mut seen = rustc_hash::FxHashSet::default();
        reqs.iter()
            .filter(|r| !self.map.contains_key(&r.key()) && seen.insert(r.key()))
            .cloned()
            .collect()
    }

    pub fn insert(&mut self, results: Vec<(CallReq, Option<Vec<u8>>)>) -> anyhow::Result<()> {
        let mut f = match &self.path {
            Some(p) => Some(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)?,
            ),
            None => None,
        };
        for (req, res) in results {
            if let Some(f) = f.as_mut() {
                writeln!(
                    f,
                    "{}\t{}",
                    req.key(),
                    res.as_ref().map_or("REVERT".to_string(), hex::encode)
                )?;
            }
            self.map.insert(req.key(), res);
        }
        Ok(())
    }

    fn get(&self, req: &CallReq) -> Option<&[u8]> {
        self.map.get(&req.key()).and_then(|r| r.as_deref())
    }

    /// First return word as an unsigned integer.
    pub fn uint(&self, to: &str, sig: &str, arg: Option<u64>, block: u64) -> Option<u128> {
        let r = self.get(&CallReq::new(to, sig, arg, block))?;
        (r.len() >= 32).then(|| u128::from_be_bytes(r[16..32].try_into().unwrap()))
    }

    /// `answer` of Chainlink's latestRoundData (second word, int256).
    pub fn eth_usd_price(&self, feed: &str, block: u64) -> Option<i128> {
        let r = self.get(&CallReq::eth_usd_price(feed, block))?;
        (r.len() >= 64).then(|| i128::from_be_bytes(r[48..64].try_into().unwrap()))
    }
}
