// Compact binary log cache, sorted by (block, logIndex).
// Record: u32 block | u32 logIndex | u8 contract | u8 ntopics | u16 0 | u32 datalen | [32] tx | topics | data

pub const CONTRACTS: [&str; 7] = [
    "0xBa5e05cb26b78eDa3A2f8e3b3814726305dcAc83", // 0 BasePaint
    "0xD68fe5b53e7E1AbeB5A4d0A6660667791f39263a", // 1 BasePaintBrush
    "0xb152f48F207d9D1C30Ff60d46E8cb8c1a5d00dEC", // 2 BasePaintBrushEvents
    "0xE6249eAfdC9C8a809fE28a5213120B1860f9a75f", // 3 BasePaintWIP
    "0xC59F475122e914aFCf31C0a9E0A2274666135e4E", // 4 BasePaintAnimation
    "0x5104482a2Ef3a03b6270D3e931eac890b86FaD01", // 5 BasePaintMetadataRegistry
    "0x75CF063a65d361527180805b244bC51c1deAb075", // 6 BasePaintSubscription
];
pub const BASE_PAINT: u8 = 0;
pub const BRUSH: u8 = 1;
pub const BRUSH_EVENTS: u8 = 2;
pub const WIP: u8 = 3;
pub const ANIMATION: u8 = 4;
pub const METADATA: u8 = 5;
pub const SUBSCRIPTION: u8 = 6;

/// Same start blocks as ponder.config.ts.
pub const START_BLOCKS: [u64; 7] = [
    0x246523, 0x246523, 15_849_605, 3_457_270, 19_092_516, 21_992_569, 30_570_812,
];
pub const FIRST_BLOCK: u64 = 0x246523;

pub const BASE_GENESIS_TS: u64 = 1686789347;

/// Base produces a block every 2 seconds since genesis.
pub fn block_timestamp(block: u64) -> u64 {
    BASE_GENESIS_TS + 2 * block
}

pub struct LogRec<'a> {
    pub block: u32,
    pub log_index: u32,
    pub contract: u8,
    pub tx: &'a [u8],
    pub topics: &'a [u8],
    pub data: &'a [u8],
    pub len: usize,
}

impl<'a> LogRec<'a> {
    pub fn parse(buf: &'a [u8]) -> LogRec<'a> {
        let block = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let log_index = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let contract = buf[8];
        let nt = buf[9] as usize;
        let dl = u32::from_le_bytes(buf[12..16].try_into().unwrap()) as usize;
        let d_off = 48 + 32 * nt;
        LogRec {
            block,
            log_index,
            contract,
            tx: &buf[16..48],
            topics: &buf[48..d_off],
            data: &buf[d_off..d_off + dl],
            len: d_off + dl,
        }
    }

    pub fn topic(&self, i: usize) -> &'a [u8] {
        &self.topics[i * 32..i * 32 + 32]
    }

    pub fn ntopics(&self) -> usize {
        self.topics.len() / 32
    }

    /// Identity used to detect reorgs: same position, same transaction, same payload.
    pub fn fingerprint(&self) -> (u32, u32, u64) {
        use std::hash::{Hash, Hasher};
        let mut h = rustc_hash::FxHasher::default();
        self.tx.hash(&mut h);
        self.topics.hash(&mut h);
        self.data.hash(&mut h);
        (self.block, self.log_index, h.finish())
    }
}

pub struct Iter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = LogRec<'a>;
    fn next(&mut self) -> Option<LogRec<'a>> {
        if self.pos >= self.buf.len() {
            return None;
        }
        let r = LogRec::parse(&self.buf[self.pos..]);
        self.pos += r.len;
        Some(r)
    }
}

pub fn iter(buf: &[u8]) -> Iter<'_> {
    Iter { buf, pos: 0 }
}

fn unhex(s: &str) -> anyhow::Result<Vec<u8>> {
    Ok(hex::decode(s.trim_start_matches("0x"))?)
}

/// Converts one `eth_getLogs` entry into a cache record.
pub fn encode_rpc_log(l: &serde_json::Value, out: &mut Vec<u8>) -> anyhow::Result<(u32, u32)> {
    let num = |k: &str| -> anyhow::Result<u64> {
        let s = l[k]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("log missing {k}"))?;
        Ok(u64::from_str_radix(s.trim_start_matches("0x"), 16)?)
    };
    let block = num("blockNumber")? as u32;
    let log_index = num("logIndex")? as u32;
    let addr = l["address"].as_str().unwrap_or_default();
    let contract = CONTRACTS
        .iter()
        .position(|a| a.eq_ignore_ascii_case(addr))
        .ok_or_else(|| anyhow::anyhow!("unexpected address {addr}"))? as u8;
    let tx = unhex(l["transactionHash"].as_str().unwrap_or_default())?;
    let topics: Vec<Vec<u8>> = l["topics"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|t| unhex(t.as_str().unwrap_or_default()))
                .collect::<Result<_, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let data = unhex(l["data"].as_str().unwrap_or("0x"))?;
    out.extend_from_slice(&block.to_le_bytes());
    out.extend_from_slice(&log_index.to_le_bytes());
    out.push(contract);
    out.push(topics.len() as u8);
    out.extend_from_slice(&[0, 0]);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&tx);
    for t in &topics {
        out.extend_from_slice(t);
    }
    out.extend_from_slice(&data);
    Ok((block, log_index))
}

/// Sorts the records of a buffer by (block, logIndex).
pub fn sort_records(buf: &[u8]) -> Vec<u8> {
    let mut recs: Vec<(u32, u32, usize, usize)> = Vec::new();
    let mut pos = 0;
    while pos < buf.len() {
        let r = LogRec::parse(&buf[pos..]);
        recs.push((r.block, r.log_index, pos, r.len));
        pos += r.len;
    }
    recs.sort_unstable_by_key(|r| (r.0, r.1));
    let mut out = Vec::with_capacity(buf.len());
    for (_, _, p, l) in recs {
        out.extend_from_slice(&buf[p..p + l]);
    }
    out
}
