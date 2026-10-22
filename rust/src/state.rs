// In-memory BasePaint state. `apply` ports the handlers in ../src/*.ts one to one;
// the comments name the TypeScript handler each branch mirrors.

use crate::calls::{CallCache, CallReq};
use crate::logs::{self, LogRec};
use crate::tables::T;
use crate::value::V;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;
use tiny_keccak::{Hasher, Keccak};

pub fn keccak(b: &[u8]) -> [u8; 32] {
    let mut h = Keccak::v256();
    h.update(b);
    let mut o = [0u8; 32];
    h.finalize(&mut o);
    o
}

pub fn checksum(a: &[u8; 20]) -> String {
    let h = hex::encode(a);
    let o = keccak(h.as_bytes());
    let mut s = String::with_capacity(42);
    s.push_str("0x");
    for (i, ch) in h.chars().enumerate() {
        let nib = (o[i / 2] >> if i % 2 == 0 { 4 } else { 0 }) & 0xf;
        s.push(if nib >= 8 {
            ch.to_ascii_uppercase()
        } else {
            ch
        });
    }
    s
}

pub fn parse_addr(s: &str) -> Option<[u8; 20]> {
    let h = s.strip_prefix("0x")?;
    if h.len() != 40 {
        return None;
    }
    hex::decode(h).ok()?.try_into().ok()
}

pub struct Topics {
    pub painted: [u8; 32],
    pub earned: [u8; 32],
    pub withdraw: [u8; 32],
    pub started: [u8; 32],
    pub t_single: [u8; 32],
    pub t_batch: [u8; 32],
    pub transfer: [u8; 32],
    pub deployed: [u8; 32],
    pub strength: [u8; 32],
    pub metadata: [u8; 32],
}

impl Topics {
    pub fn new() -> Topics {
        let k = |s: &str| keccak(s.as_bytes());
        Topics {
            painted: k("Painted(uint256,uint256,address,bytes)"),
            earned: k("ArtistsEarned(uint256,uint256)"),
            withdraw: k("ArtistWithdraw(uint256,address,uint256)"),
            started: k("Started(uint256)"),
            t_single: k("TransferSingle(address,address,address,uint256,uint256)"),
            t_batch: k("TransferBatch(address,address,address,uint256[],uint256[])"),
            transfer: k("Transfer(address,address,uint256)"),
            deployed: k("Deployed()"),
            strength: k("StrengthChanged(uint256,uint256)"),
            metadata: k("MetadataUpdated(uint256,string,uint24[],uint96,address)"),
        }
    }
}

pub const BRUSH_EVENTS_DEPLOYED_BLOCK: u32 = 15_849_605;
const ETH_USD_PRICE_FEED: &str = "0x71041dddad3595F9CEd3DcCFBe3D1F4b0a16Bb70";
const WEI_PER_ETH: i128 = 1_000_000_000_000_000_000;

#[inline]
fn word(d: &[u8], i: usize) -> &[u8] {
    &d[i * 32..i * 32 + 32]
}
#[inline]
fn u64w(w: &[u8]) -> u64 {
    u64::from_be_bytes(w[24..32].try_into().unwrap())
}
#[inline]
fn i128w(w: &[u8]) -> i128 {
    i128::from_be_bytes(w[16..32].try_into().unwrap())
}
#[inline]
fn addrw(w: &[u8]) -> [u8; 20] {
    w[12..32].try_into().unwrap()
}
fn dyn_slice(d: &[u8], i: usize) -> (usize, usize) {
    let off = u64w(word(d, i)) as usize;
    let len = u64w(&d[off..off + 32]) as usize;
    (off + 32, len)
}
fn dyn_words(d: &[u8], i: usize) -> Vec<i128> {
    let (start, len) = dyn_slice(d, i);
    (0..len)
        .map(|k| i128w(&d[start + 32 * k..start + 32 * k + 32]))
        .collect()
}

#[derive(Default, Clone)]
pub struct Global {
    pub started_at: i64,
    pub epoch_duration: i64,
    pub total_artists: i64,
    pub total_pixels: i64,
    pub total_earnings: i128,
    pub total_earnings_usd8: i128,
    pub total_withdrawals: i128,
    pub total_mints: i64,
    pub total_burns: i64,
    pub total_subscriptions: i64,
}

#[derive(Default, Clone)]
pub struct Account {
    pub addr: u32,
    pub total_pixels: i64,
    pub total_withdrawn: i128,
    pub total_earned: i128,
    pub streak: i64,
    pub longest_streak: i64,
    pub last_painted_day: Option<i64>,
    pub total_days_painted: i64,
}

#[derive(Default, Clone)]
pub struct Brush {
    pub owner: u32,
    pub strength: i64,
    pub last_used_ts: Option<i64>,
    pub last_used_day: Option<i64>,
    pub minted_ts: i64,
    pub streak: i64,
}

#[derive(Default, Clone)]
pub struct Canvas {
    pub total_mints: i64,
    pub total_burns: i64,
    pub total_earned: i128,
    pub total_earned_usd8: i128,
    pub eth_usd8: i128,
    pub total_artists: i64,
    pub pixels: i64,
    pub name: Option<String>,
    pub palette: Option<String>,
    pub size: Option<i64>,
    pub proposer: Option<u32>,
}

pub struct Contribution {
    pub day: u32,
    pub acct: u32,
    pub pixels: i64,
}

pub struct Usage {
    pub day: u32,
    pub brush: u32,
    pub pixels: i64,
}

pub struct Stroke {
    pub id: u64,
    pub day: u32,
    pub acct: u32,
    pub brush: u32,
    pub data_off: u64,
    pub data_len: u32,
    pub pixels: i64,
    pub tx: [u8; 32],
    pub ts: u32,
}

pub struct Withdrawal {
    pub day: u32,
    pub acct: u32,
    pub amount: i128,
    pub ts: u32,
}

pub struct Balance {
    pub contract: u8,
    pub owner: u32,
    pub token: u64,
    pub value: i64,
}

pub struct TotalBalance {
    pub contract: u8,
    pub owner: u32,
    pub value: i64,
}

#[derive(Default)]
pub struct State {
    pub addr_ix: FxHashMap<[u8; 20], u32>,
    pub addrs: Vec<[u8; 20]>,
    names: Vec<std::sync::OnceLock<String>>, // checksummed spelling, computed on first use
    pub account_of: Vec<u32>,                // address index -> account row + 1 (0 = none)
    pub accounts: Vec<Account>,
    pub global: Option<Global>,
    pub brushes: Vec<Option<Brush>>,
    pub canvases: Vec<Option<Canvas>>,
    pub animations: Vec<Option<i64>>,
    pub contributions: Vec<Contribution>,
    pub contribution_ix: FxHashMap<(u32, u32), u32>,
    pub usages: Vec<Usage>,
    pub usage_ix: FxHashMap<(u32, u32), u32>,
    pub strokes: Vec<Stroke>,
    pub pixels: Vec<u8>,
    pub withdrawals: Vec<Withdrawal>,
    pub withdrawal_ix: FxHashMap<(u32, u32), u32>,
    pub balances: Vec<Balance>,
    pub balance_ix: FxHashMap<(u8, u32, u64), u32>,
    pub total_balances: Vec<TotalBalance>,
    pub total_balance_ix: FxHashMap<(u8, u32), u32>,

    // Secondary indexes over columns that never change after insert.
    pub by_day_contribution: FxHashMap<u32, Vec<u32>>,
    pub by_acct_contribution: FxHashMap<u32, Vec<u32>>,
    pub by_day_usage: FxHashMap<u32, Vec<u32>>,
    pub by_brush_usage: FxHashMap<u32, Vec<u32>>,
    pub by_day_stroke: FxHashMap<u32, Vec<u32>>,
    pub by_acct_stroke: FxHashMap<u32, Vec<u32>>,
    pub by_brush_stroke: FxHashMap<u32, Vec<u32>>,
    pub by_tx_stroke: FxHashMap<[u8; 32], Vec<u32>>,
    pub by_day_withdrawal: FxHashMap<u32, Vec<u32>>,
    pub by_acct_withdrawal: FxHashMap<u32, Vec<u32>>,
    pub by_owner_balance: FxHashMap<u32, Vec<u32>>,
    pub by_token_balance: FxHashMap<u64, Vec<u32>>,
    pub by_owner_total_balance: FxHashMap<u32, Vec<u32>>,

    /// Token ids that do not fit in 63 bits (WIP token ids are hashes).
    pub big_tokens: Vec<[u8; 32]>,
    pub big_token_ix: FxHashMap<[u8; 32], u64>,

    /// Last block fully indexed (all its logs applied).
    pub synced_to: u64,
    /// Row ids of text-keyed tables in primary key (collation) order.
    pub pk_order: FxHashMap<T, Vec<u32>>,
    pub contract_names: Vec<String>,
    contract_addrs: Vec<[u8; 20]>,
}

/// Decimal digits of `n`, written into `buf`.
fn dec(mut n: u64, buf: &mut [u8; 20]) -> &[u8] {
    let mut i = 20;
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            return &buf[i..];
        }
    }
}

fn grow<X: Default>(v: &mut Vec<X>, i: usize) {
    if v.len() <= i {
        v.resize_with(i + 1, X::default);
    }
}

fn push_ix<K: std::hash::Hash + Eq>(m: &mut FxHashMap<K, Vec<u32>>, k: K, rid: u32) {
    m.entry(k).or_default().push(rid);
}

impl State {
    pub fn new() -> State {
        let mut st = State {
            contract_names: logs::CONTRACTS.iter().map(|s| s.to_string()).collect(),
            contract_addrs: logs::CONTRACTS
                .iter()
                .map(|s| parse_addr(s).unwrap())
                .collect(),
            ..Default::default()
        };
        st.ix([0u8; 20]);
        // BasePaint:setup
        st.global = Some(Global::default());
        st
    }

    #[inline]
    pub fn ix(&mut self, a: [u8; 20]) -> u32 {
        if let Some(&i) = self.addr_ix.get(&a) {
            return i;
        }
        let n = self.addrs.len() as u32;
        self.addr_ix.insert(a, n);
        self.addrs.push(a);
        self.names.push(std::sync::OnceLock::new());
        self.account_of.push(0);
        n
    }

    pub fn addr_of(&self, s: &str) -> Option<u32> {
        let i = *self.addr_ix.get(&parse_addr(s)?)?;
        // Ponder compares text ids exactly, so only the checksummed spelling matches.
        (self.name_str(i) == s).then_some(i)
    }

    pub fn name_str(&self, a: u32) -> &str {
        self.names[a as usize].get_or_init(|| checksum(&self.addrs[a as usize]))
    }

    fn account_mut(&mut self, addr: u32) -> Option<&mut Account> {
        match self.account_of[addr as usize] {
            0 => None,
            r => Some(&mut self.accounts[r as usize - 1]),
        }
    }

    /// Compact token id: the value itself below 2^63, else a tagged index into big_tokens.
    fn tok(&mut self, w: &[u8]) -> u64 {
        if w[..24].iter().all(|b| *b == 0) && w[24] & 0x80 == 0 {
            return u64w(w);
        }
        let k: [u8; 32] = w.try_into().unwrap();
        if let Some(&i) = self.big_token_ix.get(&k) {
            return i;
        }
        let i = (1u64 << 63) | self.big_tokens.len() as u64;
        self.big_tokens.push(k);
        self.big_token_ix.insert(k, i);
        i
    }

    pub fn token_value(&self, t: u64) -> ethnum::U256 {
        if t >> 63 == 0 {
            ethnum::U256::from(t)
        } else {
            ethnum::U256::from_be_bytes(self.big_tokens[(t & !(1u64 << 63)) as usize])
        }
    }

    fn track(&mut self, c: u8, from: u32, to: u32, id: u64, value: i64) {
        // utils.ts trackBalance
        if from != 0 {
            self.add_balance(c, from, id, -value);
        }
        if to != 0 {
            self.add_balance(c, to, id, value);
        }
        if from != 0 {
            self.add_total_balance(c, from, -value);
        }
        if to != 0 {
            self.add_total_balance(c, to, value);
        }
    }

    fn add_balance(&mut self, c: u8, owner: u32, token: u64, delta: i64) {
        match self.balance_ix.get(&(c, owner, token)) {
            Some(&r) => self.balances[r as usize].value += delta,
            None => {
                let r = self.balances.len() as u32;
                self.balances.push(Balance {
                    contract: c,
                    owner,
                    token,
                    value: delta,
                });
                self.balance_ix.insert((c, owner, token), r);
                push_ix(&mut self.by_owner_balance, owner, r);
                push_ix(&mut self.by_token_balance, token, r);
            }
        }
    }

    fn add_total_balance(&mut self, c: u8, owner: u32, delta: i64) {
        match self.total_balance_ix.get(&(c, owner)) {
            Some(&r) => self.total_balances[r as usize].value += delta,
            None => {
                let r = self.total_balances.len() as u32;
                self.total_balances.push(TotalBalance {
                    contract: c,
                    owner,
                    value: delta,
                });
                self.total_balance_ix.insert((c, owner), r);
                push_ix(&mut self.by_owner_total_balance, owner, r);
            }
        }
    }

    /// Contract reads the handlers will make for these logs, given the current state.
    pub fn needed_calls<'a>(
        &self,
        recs: impl Iterator<Item = LogRec<'a>>,
        tp: &Topics,
    ) -> Vec<CallReq> {
        let mut out = Vec::new();
        let mut days: FxHashSet<u64> = FxHashSet::default();
        let mut brushes: Vec<u64> = self
            .brushes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.is_some())
            .map(|(i, _)| i as u64)
            .collect();
        let mut brush_set: FxHashSet<u64> = brushes.iter().copied().collect();
        for r in recs {
            if r.ntopics() == 0 || (r.block as u64) < logs::START_BLOCKS[r.contract as usize] {
                continue;
            }
            let t0 = r.topic(0);
            if r.contract == logs::BASE_PAINT && t0 == tp.painted {
                let day = u64w(r.topic(1));
                let exists = self.canvases.get(day as usize).is_some_and(|c| c.is_some());
                if !exists && days.insert(day) {
                    out.push(CallReq::eth_usd_price(ETH_USD_PRICE_FEED, r.block as u64));
                }
            } else if r.contract == logs::BASE_PAINT && t0 == tp.started {
                out.push(CallReq::new(
                    logs::CONTRACTS[0],
                    "epochDuration()",
                    None,
                    r.block as u64,
                ));
            } else if r.contract == logs::BRUSH && t0 == tp.transfer {
                let token = u64w(r.topic(3));
                if addrw(r.topic(1)) == [0u8; 20] && r.block >= BRUSH_EVENTS_DEPLOYED_BLOCK {
                    out.push(CallReq::new(
                        logs::CONTRACTS[1],
                        "strengths(uint256)",
                        Some(token),
                        r.block as u64,
                    ));
                }
                if brush_set.insert(token) {
                    brushes.push(token);
                }
            } else if r.contract == logs::BRUSH_EVENTS && t0 == tp.deployed {
                for &b in &brushes {
                    out.push(CallReq::new(
                        logs::CONTRACTS[1],
                        "strengths(uint256)",
                        Some(b),
                        r.block as u64,
                    ));
                }
            }
        }
        out
    }

    pub fn apply(&mut self, r: &LogRec, tp: &Topics, calls: &CallCache) {
        if r.ntopics() == 0 || (r.block as u64) < logs::START_BLOCKS[r.contract as usize] {
            return;
        }
        let t0 = r.topic(0);
        let d = r.data;
        let ts = logs::block_timestamp(r.block as u64) as i64;
        let c = r.contract;

        if c == logs::BASE_PAINT && t0 == tp.painted {
            // BasePaint:Painted
            let day = u64w(r.topic(1)) as u32;
            let token = u64w(word(d, 0)) as u32;
            let author = self.ix(addrw(word(d, 1)));
            let (pstart, plen) = dyn_slice(d, 2);
            let px = (plen / 3) as i64;

            let had_account = self.account_of[author as usize] != 0;
            let is_new_artist = self
                .account_mut(author)
                .map_or(true, |a| a.total_pixels == 0);
            let had_contribution = self.contribution_ix.contains_key(&(day, author));

            if let Some(Some(b)) = self.brushes.get_mut(token as usize) {
                let mut s = b.streak;
                if b.last_used_day == Some(day as i64 - 1) {
                    s += 1;
                }
                if b.last_used_day.map_or(true, |l| l < day as i64 - 1) {
                    s = 1;
                }
                b.last_used_day = Some(day as i64);
                b.last_used_ts = Some(ts);
                b.streak = s;
            }

            grow(&mut self.canvases, day as usize);
            let canvas_exists = self.canvases[day as usize].is_some();
            let eth = if canvas_exists {
                self.canvases[day as usize].as_ref().unwrap().eth_usd8
            } else {
                // getEthUsdPriceAtStart8: Chainlink answer, else the previous day's price, else 0
                match calls.eth_usd_price(ETH_USD_PRICE_FEED, r.block as u64) {
                    Some(p) if p > 0 => p,
                    _ => day
                        .checked_sub(1)
                        .and_then(|p| self.canvases.get(p as usize))
                        .and_then(|c| c.as_ref())
                        .map_or(0, |c| c.eth_usd8),
                }
            };
            if !canvas_exists && day >= 2 {
                // Split the finished canvas's earnings between its painters.
                if let Some(Some(prev)) = self.canvases.get((day - 2) as usize) {
                    let (te, pc) = (prev.total_earned, prev.pixels);
                    if let Some(rows) = self.by_day_contribution.get(&(day - 2)) {
                        for &rid in rows {
                            let ct = &self.contributions[rid as usize];
                            let earnings =
                                ct.pixels as i128 * te / if pc == 0 { 1 } else { pc as i128 };
                            if let r @ 1.. = self.account_of[ct.acct as usize] {
                                self.accounts[r as usize - 1].total_earned += earnings;
                            }
                        }
                    }
                }
            }
            match self.contribution_ix.get(&(day, author)) {
                Some(&rid) => self.contributions[rid as usize].pixels += px,
                None => {
                    let rid = self.contributions.len() as u32;
                    self.contributions.push(Contribution {
                        day,
                        acct: author,
                        pixels: px,
                    });
                    self.contribution_ix.insert((day, author), rid);
                    push_ix(&mut self.by_day_contribution, day, rid);
                    push_ix(&mut self.by_acct_contribution, author, rid);
                }
            }
            match &mut self.canvases[day as usize] {
                Some(cv) => {
                    cv.pixels += px;
                    if !had_contribution {
                        cv.total_artists += 1;
                    }
                }
                none => {
                    *none = Some(Canvas {
                        eth_usd8: eth,
                        pixels: px,
                        total_artists: 1,
                        ..Default::default()
                    })
                }
            }
            match self.usage_ix.get(&(day, token)) {
                Some(&rid) => self.usages[rid as usize].pixels += px,
                None => {
                    let rid = self.usages.len() as u32;
                    self.usages.push(Usage {
                        day,
                        brush: token,
                        pixels: px,
                    });
                    self.usage_ix.insert((day, token), rid);
                    push_ix(&mut self.by_day_usage, day, rid);
                    push_ix(&mut self.by_brush_usage, token, rid);
                }
            }
            if had_account {
                let a = self.account_mut(author).unwrap();
                let mut s = a.streak;
                if a.last_painted_day == Some(day as i64 - 1) {
                    s += 1;
                }
                if a.last_painted_day.map_or(true, |l| l < day as i64 - 1) {
                    s = 1;
                }
                a.streak = s;
                a.longest_streak = a.longest_streak.max(s);
                a.last_painted_day = Some(day as i64);
                a.total_pixels += px;
                a.total_days_painted += if had_contribution { 0 } else { 1 };
            }
            let rid = self.strokes.len() as u32;
            let data_off = self.pixels.len() as u64;
            self.pixels.extend_from_slice(&d[pstart..pstart + plen]);
            self.strokes.push(Stroke {
                id: r.block as u64 * 100_000 + r.log_index as u64,
                day,
                acct: author,
                brush: token,
                data_off,
                data_len: plen as u32,
                pixels: px,
                tx: r.tx.try_into().unwrap(),
                ts: ts as u32,
            });
            push_ix(&mut self.by_day_stroke, day, rid);
            push_ix(&mut self.by_acct_stroke, author, rid);
            push_ix(&mut self.by_brush_stroke, token, rid);
            push_ix(&mut self.by_tx_stroke, r.tx.try_into().unwrap(), rid);
            if let Some(g) = self.global.as_mut() {
                g.total_pixels += px;
                if is_new_artist {
                    g.total_artists += 1;
                }
            }
        } else if c == logs::BASE_PAINT && t0 == tp.earned {
            // BasePaint:ArtistsEarned
            let day = u64w(r.topic(1)) as usize;
            let amount = i128w(word(d, 0));
            let mut usd = 0;
            if let Some(Some(cv)) = self.canvases.get_mut(day) {
                usd = amount * cv.eth_usd8 / WEI_PER_ETH;
                cv.total_earned += amount;
                cv.total_earned_usd8 += usd;
            }
            if let Some(g) = self.global.as_mut() {
                g.total_earnings += amount;
                g.total_earnings_usd8 += usd;
            }
        } else if c == logs::BASE_PAINT && t0 == tp.withdraw {
            // BasePaint:ArtistWithdraw
            let day = u64w(r.topic(1)) as u32;
            let author = self.ix(addrw(word(d, 0)));
            let amount = i128w(word(d, 1));
            if let Some(a) = self.account_mut(author) {
                a.total_withdrawn += amount;
            }
            let rid = self.withdrawals.len() as u32;
            self.withdrawals.push(Withdrawal {
                day,
                acct: author,
                amount,
                ts: ts as u32,
            });
            self.withdrawal_ix.insert((day, author), rid);
            push_ix(&mut self.by_day_withdrawal, day, rid);
            push_ix(&mut self.by_acct_withdrawal, author, rid);
            if let Some(g) = self.global.as_mut() {
                g.total_withdrawals += amount;
            }
        } else if c == logs::BASE_PAINT && t0 == tp.started {
            // BasePaint:Started
            let epoch = calls
                .uint(logs::CONTRACTS[0], "epochDuration()", None, r.block as u64)
                .unwrap_or(0);
            if let Some(g) = self.global.as_mut() {
                g.started_at = ts;
                g.epoch_duration = epoch as i64;
            }
        } else if (c == logs::BASE_PAINT || c == logs::ANIMATION || c == logs::SUBSCRIPTION)
            && (t0 == tp.t_single || t0 == tp.t_batch)
        {
            let from = self.ix(addrw(r.topic(2)));
            let to = self.ix(addrw(r.topic(3)));
            let batch = t0 == tp.t_batch;
            let pairs: Vec<(u64, i64)> = if batch {
                let (ids, n) = dyn_slice(d, 0);
                let (vals, _) = dyn_slice(d, 1);
                (0..n)
                    .map(|k| {
                        (
                            self.tok(&d[ids + 32 * k..ids + 32 * k + 32]),
                            u64w(&d[vals + 32 * k..vals + 32 * k + 32]) as i64,
                        )
                    })
                    .collect()
            } else {
                vec![(self.tok(word(d, 0)), u64w(word(d, 1)) as i64)]
            };
            for &(id, v) in &pairs {
                self.track(c, from, to, id, v);
            }
            match c {
                logs::BASE_PAINT => {
                    // BasePaint:TransferSingle / TransferBatch
                    if from == 0 {
                        for &(id, v) in &pairs {
                            let exists = matches!(self.canvases.get(id as usize), Some(Some(_)));
                            if let Some(Some(cv)) = self.canvases.get_mut(id as usize) {
                                cv.total_mints += v;
                            }
                            if exists || batch {
                                if let Some(g) = self.global.as_mut() {
                                    g.total_mints += v;
                                }
                            }
                        }
                    }
                }
                logs::ANIMATION => {
                    // BasePaintAnimation:TransferSingle / TransferBatch
                    if from == 0 {
                        for &(id, v) in &pairs {
                            grow(&mut self.animations, id as usize);
                            *self.animations[id as usize].get_or_insert(0) += v;
                            if let Some(Some(cv)) = self.canvases.get_mut(id as usize) {
                                cv.total_burns += 2 * v;
                            }
                            if let Some(g) = self.global.as_mut() {
                                g.total_burns += 2 * v;
                            }
                        }
                    }
                }
                _ => {
                    // BasePaintSubscription:TransferSingle / TransferBatch
                    let sum: i64 = pairs.iter().map(|p| p.1).sum();
                    let delta = if from == 0 {
                        sum
                    } else if to == 0 {
                        -sum
                    } else {
                        0
                    };
                    if delta != 0 {
                        if let Some(g) = self.global.as_mut() {
                            g.total_subscriptions += delta;
                        }
                    }
                }
            }
        } else if (c == logs::BRUSH || c == logs::WIP) && t0 == tp.transfer {
            // BasePaintBrush:Transfer / BasePaintWIP:Transfer
            let from = self.ix(addrw(r.topic(1)));
            let to = self.ix(addrw(r.topic(2)));
            let token = u64w(r.topic(3));
            let tok = self.tok(r.topic(3));
            self.track(c, from, to, tok, 1);
            if c == logs::BRUSH {
                let strength = if from == 0 && r.block >= BRUSH_EVENTS_DEPLOYED_BLOCK {
                    calls
                        .uint(
                            logs::CONTRACTS[1],
                            "strengths(uint256)",
                            Some(token),
                            r.block as u64,
                        )
                        .unwrap_or(0) as i64
                } else {
                    0
                };
                grow(&mut self.brushes, token as usize);
                match &mut self.brushes[token as usize] {
                    Some(b) => b.owner = to,
                    none => {
                        *none = Some(Brush {
                            owner: to,
                            strength,
                            minted_ts: ts,
                            ..Default::default()
                        })
                    }
                }
                if self.account_of[to as usize] == 0 {
                    self.accounts.push(Account {
                        addr: to,
                        ..Default::default()
                    });
                    self.account_of[to as usize] = self.accounts.len() as u32;
                }
            }
        } else if c == logs::BRUSH_EVENTS && t0 == tp.deployed {
            // BasePaintBrushEvents:Deployed
            let block = r.block as u64;
            for (id, b) in self.brushes.iter_mut().enumerate() {
                if let Some(b) = b {
                    b.strength = calls
                        .uint(
                            logs::CONTRACTS[1],
                            "strengths(uint256)",
                            Some(id as u64),
                            block,
                        )
                        .unwrap_or(0) as i64;
                }
            }
        } else if c == logs::BRUSH_EVENTS && t0 == tp.strength {
            // BasePaintBrushEvents:StrengthChanged
            if let Some(Some(b)) = self.brushes.get_mut(u64w(r.topic(1)) as usize) {
                b.strength = u64w(word(d, 0)) as i64;
            }
        } else if c == logs::METADATA && t0 == tp.metadata {
            // BasePaintMetadataRegistry:MetadataUpdated
            let id = u64w(r.topic(1)) as usize;
            let (ns, nl) = dyn_slice(d, 0);
            let name = String::from_utf8_lossy(&d[ns..ns + nl]).into_owned();
            let palette = dyn_words(d, 1)
                .iter()
                .map(|c| format!("#{:06X}", c))
                .collect::<Vec<_>>()
                .join(",");
            let size = u64w(word(d, 2)) as i64;
            let proposer = self.ix(addrw(word(d, 3)));
            if let Some(Some(cv)) = self.canvases.get_mut(id) {
                cv.name = Some(name);
                cv.palette = Some(palette);
                cv.size = Some(size);
                cv.proposer = Some(proposer);
            }
        }
    }

    // ---- Generic row access for the GraphQL layer ----

    /// Number of row slots; some slots may be empty (`exists` is false).
    pub fn slots(&self, t: T) -> usize {
        match t {
            T::Global => self.global.is_some() as usize,
            T::Account => self.accounts.len(),
            T::Brush => self.brushes.len(),
            T::Contribution => self.contributions.len(),
            T::Usage => self.usages.len(),
            T::Stroke => self.strokes.len(),
            T::Withdrawal => self.withdrawals.len(),
            T::Canvas => self.canvases.len(),
            T::Animation => self.animations.len(),
            T::Balance => self.balances.len(),
            T::TotalBalance => self.total_balances.len(),
        }
    }

    pub fn exists(&self, t: T, rid: u32) -> bool {
        let r = rid as usize;
        match t {
            T::Brush => self.brushes.get(r).is_some_and(|b| b.is_some()),
            T::Canvas => self.canvases.get(r).is_some_and(|b| b.is_some()),
            T::Animation => self.animations.get(r).is_some_and(|b| b.is_some()),
            _ => r < self.slots(t),
        }
    }

    /// True when slot order equals primary key order (numeric single-column keys).
    pub fn natural_pk_order(t: T) -> bool {
        matches!(
            t,
            T::Global | T::Brush | T::Stroke | T::Canvas | T::Animation
        )
    }

    /// Columns whose values never decrease with row id (rows are appended in block order).
    pub fn monotonic(t: T, col: usize) -> bool {
        matches!(
            (t, col),
            (T::Stroke, 0) | (T::Stroke, 7) | (T::Withdrawal, 4)
        ) || (Self::natural_pk_order(t) && col == 0)
    }

    pub const TEXT_PK_TABLES: [T; 6] = [
        T::Account,
        T::Contribution,
        T::Usage,
        T::Withdrawal,
        T::Balance,
        T::TotalBalance,
    ];

    /// Compares two rows by primary key in the same order as value::collate on the id strings,
    /// without building the strings. Ids made of fixed-width hex addresses never tie at the
    /// first collation level, so comparing the raw bytes is enough for them.
    pub fn pk_cmp(&self, t: T, a: u32, b: u32) -> Ordering {
        let (a, b) = (a as usize, b as usize);
        let addr = |i: u32| &self.addrs[i as usize];
        let contract = |c: u8| &self.contract_addrs[c as usize];
        match t {
            T::Account => addr(self.accounts[a].addr).cmp(addr(self.accounts[b].addr)),
            T::Contribution => {
                let (x, y) = (&self.contributions[a], &self.contributions[b]);
                x.day
                    .cmp(&y.day)
                    .then_with(|| addr(x.acct).cmp(addr(y.acct)))
            }
            T::Usage => {
                // "day_brush": digits compare first, then the earlier underscore wins.
                let (x, y) = (&self.usages[a], &self.usages[b]);
                let (mut bx, mut by) = ([0u8; 20], [0u8; 20]);
                let (dx, dy) = (dec(x.day as u64, &mut bx), dec(y.day as u64, &mut by));
                let (lx, ly) = (dx.len(), dy.len());
                let (mut cx, mut cy) = ([0u8; 20], [0u8; 20]);
                let (ex, ey) = (dec(x.brush as u64, &mut cx), dec(y.brush as u64, &mut cy));
                dx.iter()
                    .chain(ex.iter())
                    .cmp(dy.iter().chain(ey.iter()))
                    .then(lx.cmp(&ly))
            }
            T::Withdrawal => {
                // "day_0xaddr": day digits, then the lowercase address (hex order is byte order).
                let (x, y) = (&self.withdrawals[a], &self.withdrawals[b]);
                let (mut bx, mut by) = ([0u8; 20], [0u8; 20]);
                let (dx, dy) = (dec(x.day as u64, &mut bx), dec(y.day as u64, &mut by));
                let hx = |s: &[u8; 20]| {
                    let mut out = [0u8; 42];
                    out[..2].copy_from_slice(b"0x");
                    for (i, b) in s.iter().enumerate() {
                        out[2 + 2 * i] = b"0123456789abcdef"[(b >> 4) as usize];
                        out[3 + 2 * i] = b"0123456789abcdef"[(b & 15) as usize];
                    }
                    out
                };
                let (ax, ay) = (hx(addr(x.acct)), hx(addr(y.acct)));
                dx.iter().chain(ax.iter()).cmp(dy.iter().chain(ay.iter()))
            }
            T::Balance => {
                let (x, y) = (&self.balances[a], &self.balances[b]);
                contract(x.contract)
                    .cmp(contract(y.contract))
                    .then_with(|| addr(x.owner).cmp(addr(y.owner)))
                    .then_with(|| self.token_digits_cmp(x.token, y.token))
            }
            T::TotalBalance => {
                let (x, y) = (&self.total_balances[a], &self.total_balances[b]);
                contract(x.contract)
                    .cmp(contract(y.contract))
                    .then_with(|| addr(x.owner).cmp(addr(y.owner)))
            }
            _ => a.cmp(&b),
        }
    }

    /// Token ids compare as decimal strings (that is how they appear inside balance ids).
    fn token_digits_cmp(&self, x: u64, y: u64) -> Ordering {
        if x >> 63 == 0 && y >> 63 == 0 {
            let (mut bx, mut by) = ([0u8; 20], [0u8; 20]);
            dec(x, &mut bx).cmp(dec(y, &mut by))
        } else {
            self.token_value(x)
                .to_string()
                .cmp(&self.token_value(y).to_string())
        }
    }

    /// Brings `pk_order` up to date with rows inserted since the last call.
    pub fn refresh_pk_order(&mut self) {
        for t in Self::TEXT_PK_TABLES {
            let n = self.slots(t);
            let mut order = self.pk_order.remove(&t).unwrap_or_default();
            let have = order.len();
            if n - have > have / 8 + 1000 {
                order = (0..n as u32).collect();
                order.sort_unstable_by(|&a, &b| self.pk_cmp(t, a, b));
            } else {
                for r in have as u32..n as u32 {
                    let pos = order.partition_point(|&o| self.pk_cmp(t, o, r) == Ordering::Less);
                    order.insert(pos, r);
                }
            }
            self.pk_order.insert(t, order);
        }
    }

    fn name(&self, a: u32) -> V {
        V::Text(self.name_str(a).to_string())
    }

    pub fn get(&self, t: T, rid: u32, col: usize) -> V {
        let r = rid as usize;
        let opt = |o: Option<i64>| o.map_or(V::Null, V::Int);
        match t {
            T::Global => {
                let g = self.global.as_ref().unwrap();
                match col {
                    0 => V::Int(1),
                    1 => V::Int(g.started_at),
                    2 => V::Int(g.epoch_duration),
                    3 => V::Int(g.total_artists),
                    4 => V::Int(g.total_pixels),
                    5 => V::big(g.total_earnings),
                    6 => V::big(g.total_earnings_usd8),
                    7 => V::big(g.total_withdrawals),
                    8 => V::Int(g.total_mints),
                    9 => V::Int(g.total_burns),
                    _ => V::Int(g.total_subscriptions),
                }
            }
            T::Account => {
                let a = &self.accounts[r];
                match col {
                    0 => self.name(a.addr),
                    1 => V::Int(a.total_pixels),
                    2 => V::big(a.total_withdrawn),
                    3 => V::big(a.total_earned),
                    4 => V::Int(a.streak),
                    5 => V::Int(a.longest_streak),
                    6 => opt(a.last_painted_day),
                    _ => V::Int(a.total_days_painted),
                }
            }
            T::Brush => {
                let b = self.brushes[r].as_ref().unwrap();
                match col {
                    0 => V::Int(r as i64),
                    1 => self.name(b.owner),
                    2 => V::Int(b.strength),
                    3 => opt(b.last_used_ts),
                    4 => opt(b.last_used_day),
                    5 => V::Int(b.minted_ts),
                    _ => V::Int(b.streak),
                }
            }
            T::Contribution => {
                let c = &self.contributions[r];
                match col {
                    0 => V::Text(format!("{}_{}", c.day, self.name_str(c.acct))),
                    1 => self.name(c.acct),
                    2 => V::Int(c.day as i64),
                    _ => V::Int(c.pixels),
                }
            }
            T::Usage => {
                let u = &self.usages[r];
                match col {
                    0 => V::Text(format!("{}_{}", u.day, u.brush)),
                    1 => V::Int(u.brush as i64),
                    2 => V::Int(u.day as i64),
                    _ => V::Int(u.pixels),
                }
            }
            T::Stroke => {
                let s = &self.strokes[r];
                match col {
                    0 => V::big(s.id as i128),
                    1 => V::Int(s.day as i64),
                    2 => self.name(s.acct),
                    3 => V::Int(s.brush as i64),
                    4 => {
                        let data = &self.pixels
                            [s.data_off as usize..s.data_off as usize + s.data_len as usize];
                        let mut out = String::with_capacity(2 + data.len() * 2);
                        out.push_str("0x");
                        out.push_str(&hex::encode(data));
                        V::Text(out)
                    }
                    5 => V::Int(s.pixels),
                    6 => V::Text(format!("0x{}", hex::encode(s.tx))),
                    _ => V::Int(s.ts as i64),
                }
            }
            T::Withdrawal => {
                let w = &self.withdrawals[r];
                match col {
                    0 => V::Text(format!("{}_{}", w.day, self.name_str(w.acct))),
                    1 => self.name(w.acct),
                    2 => V::Int(w.day as i64),
                    3 => V::big(w.amount),
                    _ => V::Int(w.ts as i64),
                }
            }
            T::Canvas => {
                let c = self.canvases[r].as_ref().unwrap();
                let text = |s: &Option<String>| s.as_ref().map_or(V::Null, |s| V::Text(s.clone()));
                match col {
                    0 => V::Int(r as i64),
                    1 => V::Int(c.total_mints),
                    2 => V::Int(c.total_burns),
                    3 => V::big(c.total_earned),
                    4 => V::big(c.total_earned_usd8),
                    5 => V::big(c.eth_usd8),
                    6 => V::Int(c.total_artists),
                    7 => V::Int(c.pixels),
                    8 => text(&c.name),
                    9 => text(&c.palette),
                    10 => opt(c.size),
                    _ => c.proposer.map_or(V::Null, |p| self.name(p)),
                }
            }
            T::Animation => match col {
                0 => V::Int(r as i64),
                _ => V::Int(self.animations[r].unwrap()),
            },
            T::Balance => {
                let b = &self.balances[r];
                let contract = &self.contract_names[b.contract as usize];
                match col {
                    0 => V::Text(format!(
                        "{}_{}_{}",
                        contract,
                        self.name_str(b.owner),
                        self.token_value(b.token)
                    )),
                    1 => self.name(b.owner),
                    2 => V::Text(contract.clone()),
                    3 => V::Big(self.token_value(b.token)),
                    _ => V::Int(b.value),
                }
            }
            T::TotalBalance => {
                let b = &self.total_balances[r];
                let contract = &self.contract_names[b.contract as usize];
                match col {
                    0 => V::Text(format!("{}_{}", contract, self.name_str(b.owner))),
                    1 => self.name(b.owner),
                    2 => V::Text(contract.clone()),
                    _ => V::Int(b.value),
                }
            }
        }
    }

    fn contract_ix(&self, s: &str) -> Option<u8> {
        self.contract_names
            .iter()
            .position(|c| c == s)
            .map(|i| i as u8)
    }

    /// Looks a row up by its primary key values (in TableDef::pk order).
    pub fn find(&self, t: T, pk: &[V]) -> Option<u32> {
        let int = |v: &V| match v {
            V::Int(i) => Some(*i),
            V::Big(b) => (*b <= ethnum::U256::from(i64::MAX as u64)).then(|| b.as_i64()),
            _ => None,
        };
        let text = |v: &V| match v {
            V::Text(s) => Some(s.clone()),
            _ => None,
        };
        let rid = match t {
            T::Global => (int(&pk[0])? == 1 && self.global.is_some()).then_some(0)?,
            T::Account => {
                let a = self.addr_of(&text(&pk[0])?)?;
                match self.account_of[a as usize] {
                    0 => return None,
                    r => r - 1,
                }
            }
            T::Brush | T::Canvas | T::Animation => u32::try_from(int(&pk[0])?).ok()?,
            T::Contribution => {
                let day = u32::try_from(int(&pk[0])?).ok()?;
                *self
                    .contribution_ix
                    .get(&(day, self.addr_of(&text(&pk[1])?)?))?
            }
            T::Usage => {
                let s = text(&pk[0])?;
                let (day, brush) = s.split_once('_')?;
                let rid = *self
                    .usage_ix
                    .get(&(day.parse().ok()?, brush.parse().ok()?))?;
                (self.get(t, rid, 0) == V::Text(s)).then_some(rid)?
            }
            T::Stroke => {
                let id = u64::try_from(int(&pk[0])?).ok()?;
                self.strokes.binary_search_by_key(&id, |s| s.id).ok()? as u32
            }
            T::Withdrawal => {
                let s = text(&pk[0])?;
                let (day, acct) = s.split_once('_')?;
                *self
                    .withdrawal_ix
                    .get(&(day.parse().ok()?, self.addr_of(acct)?))?
            }
            T::Balance => {
                let s = text(&pk[0])?;
                let mut parts = s.splitn(3, '_');
                let c = self.contract_ix(parts.next()?)?;
                let owner = self.addr_of(parts.next()?)?;
                let token = ethnum::U256::from_str_radix(parts.next()?, 10).ok()?;
                let token = if token < ethnum::U256::from(1u64 << 63) {
                    token.as_u64()
                } else {
                    *self.big_token_ix.get(&token.to_be_bytes())?
                };
                let rid = *self.balance_ix.get(&(c, owner, token))?;
                (self.get(t, rid, 0) == V::Text(s)).then_some(rid)?
            }
            T::TotalBalance => {
                let s = text(&pk[0])?;
                let (c, owner) = s.split_once('_')?;
                *self
                    .total_balance_ix
                    .get(&(self.contract_ix(c)?, self.addr_of(owner)?))?
            }
        };
        self.exists(t, rid).then_some(rid)
    }

    /// Candidate rows for `col == v` when an index exists for that column.
    pub fn index_lookup(&self, t: T, col: &str, v: &V) -> Option<Vec<u32>> {
        let day = || match v {
            V::Int(i) => u32::try_from(*i).ok(),
            _ => None,
        };
        let acct = || match v {
            V::Text(s) => self.addr_of(s),
            _ => None,
        };
        let get = |m: &FxHashMap<u32, Vec<u32>>, k: Option<u32>| {
            k.and_then(|k| m.get(&k).cloned()).unwrap_or_default()
        };
        Some(match (t, col) {
            (T::Contribution, "canvasId") => get(&self.by_day_contribution, day()),
            (T::Contribution, "accountId") => get(&self.by_acct_contribution, acct()),
            (T::Usage, "canvasId") => get(&self.by_day_usage, day()),
            (T::Usage, "brushId") => get(&self.by_brush_usage, day()),
            (T::Stroke, "canvasId") => get(&self.by_day_stroke, day()),
            (T::Stroke, "accountId") => get(&self.by_acct_stroke, acct()),
            (T::Stroke, "brushId") => get(&self.by_brush_stroke, day()),
            (T::Stroke, "tx") => match v {
                V::Text(s) => s
                    .strip_prefix("0x")
                    .and_then(|h| hex::decode(h).ok())
                    .and_then(|b| <[u8; 32]>::try_from(b).ok())
                    .filter(|_| s.chars().all(|c| !c.is_ascii_uppercase()))
                    .and_then(|k| self.by_tx_stroke.get(&k).cloned())
                    .unwrap_or_default(),
                _ => vec![],
            },
            (T::Withdrawal, "canvasId") => get(&self.by_day_withdrawal, day()),
            (T::Withdrawal, "accountId") => get(&self.by_acct_withdrawal, acct()),
            (T::Balance, "ownerId") => get(&self.by_owner_balance, acct()),
            (T::Balance, "tokenId") => {
                let tok = match v {
                    V::Big(u) if *u < ethnum::U256::from(1u64 << 63) => Some(u.as_u64()),
                    V::Big(u) => self.big_token_ix.get(&u.to_be_bytes()).copied(),
                    V::Int(i) => u64::try_from(*i).ok(),
                    _ => None,
                };
                tok.and_then(|k| self.by_token_balance.get(&k).cloned())
                    .unwrap_or_default()
            }
            (T::TotalBalance, "ownerId") => get(&self.by_owner_total_balance, acct()),
            (_, "id") if t != T::Contribution => {
                self.find(t, std::slice::from_ref(v)).into_iter().collect()
            }
            _ => return None,
        })
    }
}
