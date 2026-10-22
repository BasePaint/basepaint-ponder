// Log cache on disk, startup rebuild, and the live polling loop with reorg detection.

use crate::calls::CallCache;
use crate::gql::Db;
use crate::logs::{self, LogRec};
use crate::rpc::Rpc;
use crate::state::{State, Topics};
use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Store {
    pub dir: PathBuf,
    pub calls: CallCache,
    /// (block, logIndex, payload hash) of applied logs in the reorg window.
    tail: VecDeque<(u32, u32, u64)>,
    pub reorg_window: u64,
}

impl Store {
    fn logs_path(&self) -> PathBuf {
        self.dir.join("logs.bin")
    }
    fn synced_path(&self) -> PathBuf {
        self.dir.join("synced_to")
    }

    pub fn open(dir: PathBuf, reorg_window: u64) -> Result<Store> {
        std::fs::create_dir_all(&dir)?;
        let calls = CallCache::load(&dir.join("calls.tsv"))?;
        Ok(Store {
            dir,
            calls,
            tail: VecDeque::new(),
            reorg_window,
        })
    }

    pub fn synced_to(&self) -> u64 {
        std::fs::read_to_string(self.synced_path())
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(logs::FIRST_BLOCK - 1)
    }

    /// Cuts the cache file just before the first record at or after `block`.
    fn truncate_from(&self, block: u64) -> Result<()> {
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.logs_path())
        {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        if file.metadata()?.len() == 0 {
            return Ok(());
        }
        // SAFETY: nothing else writes the cache while the map is alive.
        let map = unsafe { memmap2::Mmap::map(&file)? };
        let mut end = 0;
        for r in logs::iter(&map) {
            if r.block as u64 >= block {
                break;
            }
            end += r.len;
        }
        if end < map.len() {
            drop(map);
            file.set_len(end as u64)?;
        }
        Ok(())
    }

    fn append(&mut self, recs: &[u8], synced_to: u64) -> Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.logs_path())?;
        f.write_all(recs)?;
        f.sync_data()?;
        std::fs::write(self.synced_path(), synced_to.to_string())?;
        for r in logs::iter(recs) {
            self.tail.push_back(r.fingerprint());
        }
        let keep_from = synced_to.saturating_sub(self.reorg_window);
        while self.tail.front().is_some_and(|f| (f.0 as u64) < keep_from) {
            self.tail.pop_front();
        }
        Ok(())
    }

    fn truncate_before(&mut self, block: u64) -> Result<()> {
        self.truncate_from(block)?;
        std::fs::write(self.synced_path(), (block - 1).to_string())?;
        self.tail.retain(|f| (f.0 as u64) < block);
        Ok(())
    }

    /// Fetches any eth_call results these logs need that are not cached yet.
    async fn prefetch_calls(
        &mut self,
        rpc: &Rpc,
        st: &State,
        recs: &[u8],
        tp: &Topics,
    ) -> Result<()> {
        let missing = self.calls.missing(&st.needed_calls(logs::iter(recs), tp));
        self.fetch_calls(rpc, missing).await
    }

    async fn fetch_calls(&mut self, rpc: &Rpc, missing: Vec<crate::calls::CallReq>) -> Result<()> {
        if !missing.is_empty() {
            let n = missing.len();
            let t = Instant::now();
            let res = rpc.eth_calls(missing).await?;
            self.calls.insert(res)?;
            eprintln!(
                "fetched {n} contract reads in {:.1}s",
                t.elapsed().as_secs_f64()
            );
        }
        Ok(())
    }

    /// Builds state from the whole cache. `rpc` fills in contract reads missing from the call cache.
    pub async fn build(&mut self, rpc: &Rpc, tp: &Topics) -> Result<State> {
        let synced_to = self.synced_to();
        // Records past synced_to were written before a crash; they are refetched.
        self.truncate_from(synced_to + 1)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(self.logs_path())?;
        let map = if file.metadata()?.len() > 0 {
            // SAFETY: the cache file is only appended to by this process while the map is alive.
            Some(unsafe { memmap2::Mmap::map(&file)? })
        } else {
            None
        };
        let buf: &[u8] = map.as_deref().unwrap_or_default();
        let mut st = State::new();
        self.prefetch_calls(rpc, &st, buf, tp).await?;
        let t = Instant::now();
        let mut n = 0;
        for r in logs::iter(buf) {
            st.apply(&r, tp, &self.calls);
            n += 1;
        }
        st.synced_to = synced_to;
        let applied = t.elapsed().as_secs_f64();
        st.refresh_pk_order();
        eprintln!(
            "indexed {n} cached logs up to block {synced_to} in {applied:.2}s, sorted text keys in {:.2}s",
            t.elapsed().as_secs_f64() - applied
        );
        self.tail = logs::iter(buf)
            .filter(|r| r.block as u64 >= synced_to.saturating_sub(self.reorg_window))
            .map(|r: LogRec| r.fingerprint())
            .collect();
        Ok(st)
    }
}

/// One polling round: fetch new logs (re-reading the reorg window), apply them.
async fn poll_once(db: &Arc<Db>, store: &mut Store, rpc: &Rpc, tp: &Topics) -> Result<bool> {
    let head = rpc.block_number().await?;
    let synced = db.state.read().synced_to;
    if head <= synced {
        return Ok(true);
    }
    // Catch up in bounded steps so a long backfill becomes visible progressively.
    let to = head.min(synced + 2_000_000);
    let from = synced
        .saturating_sub(store.reorg_window)
        .max(logs::FIRST_BLOCK - 1)
        + 1;
    let fetched = rpc.get_logs(from, to).await?;

    let mut split = 0;
    let mut seen: Vec<(u32, u32, u64)> = Vec::new();
    for r in logs::iter(&fetched) {
        if r.block as u64 > synced {
            break;
        }
        seen.push(r.fingerprint());
        split += r.len;
    }
    let applied: Vec<_> = store
        .tail
        .iter()
        .filter(|f| f.0 as u64 >= from)
        .copied()
        .collect();
    let mut new = fetched[split..].to_vec();
    if seen != applied {
        // The chain changed under blocks we already indexed: drop them and rebuild (a few seconds).
        eprintln!("reorg detected between blocks {from} and {synced}; rebuilding");
        store.truncate_before(from)?;
        let st = store.build(rpc, tp).await?;
        *db.state.write() = st;
        new = fetched;
    }

    let missing = store
        .calls
        .missing(&db.state.read().needed_calls(logs::iter(&new), tp));
    store.fetch_calls(rpc, missing).await?;
    {
        let mut st = db.state.write();
        for r in logs::iter(&new) {
            st.apply(&r, tp, &store.calls);
        }
        st.synced_to = to;
        st.refresh_pk_order();
    }
    store.append(&new, to)?;
    Ok(to == head)
}

pub async fn run(db: Arc<Db>, mut store: Store, rpc: Rpc, poll: Duration) {
    let tp = Topics::new();
    loop {
        match poll_once(&db, &mut store, &rpc, &tp).await.context("sync") {
            Ok(caught_up) => {
                if caught_up && !db.ready.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    eprintln!("caught up to block {}", db.state.read().synced_to);
                }
                if caught_up {
                    tokio::time::sleep(poll).await;
                }
            }
            Err(e) => {
                eprintln!("{e:#}");
                tokio::time::sleep(poll).await;
            }
        }
    }
}
