// Plural query semantics copied from Ponder 0.16 (src/graphql/index.ts executePluralQuery):
// where filters, orderBy + primary key tiebreak, offset, and before/after cursors encoded
// as base64(superjson) so cursors are interchangeable with the TypeScript server.

use crate::state::State;
use crate::tables::{ColType, T, TableDef};
use crate::value::{V, like};
use async_graphql::Value as G;
use base64::Engine;
use std::cmp::Ordering;

pub const DEFAULT_LIMIT: i64 = 50;
pub const MAX_LIMIT: i64 = 1000;

#[derive(Default)]
pub struct PluralArgs {
    pub where_: Option<G>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub struct Page {
    pub table: T,
    pub rows: Vec<u32>,
    pub total: Option<i64>,
    pub has_next: bool,
    pub has_prev: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[derive(Debug)]
pub enum Op {
    Eq,
    Ne,
    IsNull,
    NotNull,
    In,
    NotIn,
    Gt,
    Lt,
    Gte,
    Lte,
    Like {
        prefix: bool,
        suffix: bool,
        nocase: bool,
        negate: bool,
    },
}

#[derive(Debug)]
pub enum Cond {
    And(Vec<Cond>),
    Or(Vec<Cond>),
    Cmp {
        col: usize,
        op: Op,
        vals: Vec<V>,
        pat: String,
    },
}

const SUFFIXES: [&str; 22] = [
    "",
    "_not",
    "_in",
    "_not_in",
    "_has",
    "_not_has",
    "_gt",
    "_lt",
    "_gte",
    "_lte",
    "_contains",
    "_not_contains",
    "_starts_with",
    "_ends_with",
    "_not_starts_with",
    "_not_ends_with",
    "_contains_nocase",
    "_not_contains_nocase",
    "_starts_with_nocase",
    "_ends_with_nocase",
    "_not_starts_with_nocase",
    "_not_ends_with_nocase",
];

/// Mirrors buildWhereConditions, including its quirk that AND/OR flatten the conditions
/// of every sub-filter into one list.
pub fn compile_where(def: &TableDef, w: &G) -> Result<Vec<Cond>, String> {
    let mut out = Vec::new();
    let obj = match w {
        G::Object(o) => o,
        G::Null => return Ok(out),
        _ => return Err("Invalid where argument".into()),
    };
    for (key, raw) in obj.iter() {
        let key = key.as_str();
        if key == "AND" || key == "OR" {
            let G::List(list) = raw else {
                return Err(format!(
                    "Invalid query: Expected an array for the {key} operator. Got: {raw}"
                ));
            };
            let mut nested = Vec::new();
            for sub in list {
                nested.extend(compile_where(def, sub)?);
            }
            if !nested.is_empty() {
                out.push(if key == "AND" {
                    Cond::And(nested)
                } else {
                    Cond::Or(nested)
                });
            }
            continue;
        }
        let mut sufs: Vec<&str> = SUFFIXES.to_vec();
        sufs.sort_by_key(|s| std::cmp::Reverse(s.len()));
        let suffix = sufs.into_iter().find(|s| key.ends_with(s)).unwrap();
        let col_name = &key[..key.len() - suffix.len()];
        let col = def.col(col_name).ok_or_else(|| {
            format!("Invalid query: Where clause contains unknown column {col_name}")
        })?;
        let ty = def.cols[col].ty;
        let one = |v: &G| V::from_gql(v, ty);
        let list = |v: &G| -> Result<Vec<V>, String> {
            match v {
                G::List(l) => l.iter().map(|x| V::from_gql(x, ty)).collect(),
                other => Ok(vec![V::from_gql(other, ty)?]),
            }
        };
        let pat = |v: &G| match v {
            G::String(s) => s.clone(),
            other => other.to_string(),
        };
        let lk = |prefix, suffix, nocase, negate| Op::Like {
            prefix,
            suffix,
            nocase,
            negate,
        };
        let (op, vals, p) = match suffix {
            "" if raw == &G::Null => (Op::IsNull, vec![], String::new()),
            "" => (Op::Eq, vec![one(raw)?], String::new()),
            "_not" if raw == &G::Null => (Op::NotNull, vec![], String::new()),
            "_not" => (Op::Ne, vec![one(raw)?], String::new()),
            "_in" => (Op::In, list(raw)?, String::new()),
            "_not_in" => (Op::NotIn, list(raw)?, String::new()),
            "_gt" => (Op::Gt, vec![one(raw)?], String::new()),
            "_lt" => (Op::Lt, vec![one(raw)?], String::new()),
            "_gte" => (Op::Gte, vec![one(raw)?], String::new()),
            "_lte" => (Op::Lte, vec![one(raw)?], String::new()),
            "_contains" => (lk(true, true, false, false), vec![], pat(raw)),
            "_not_contains" => (lk(true, true, false, true), vec![], pat(raw)),
            "_starts_with" => (lk(false, true, false, false), vec![], pat(raw)),
            "_ends_with" => (lk(true, false, false, false), vec![], pat(raw)),
            "_not_starts_with" => (lk(false, true, false, true), vec![], pat(raw)),
            "_not_ends_with" => (lk(true, false, false, true), vec![], pat(raw)),
            "_contains_nocase" => (lk(true, true, true, false), vec![], pat(raw)),
            "_not_contains_nocase" => (lk(true, true, true, true), vec![], pat(raw)),
            "_starts_with_nocase" => (lk(false, true, true, false), vec![], pat(raw)),
            "_ends_with_nocase" => (lk(true, false, true, false), vec![], pat(raw)),
            "_not_starts_with_nocase" => (lk(false, true, true, true), vec![], pat(raw)),
            "_not_ends_with_nocase" => (lk(true, false, true, true), vec![], pat(raw)),
            other => return Err(format!("Unsupported filter suffix {other}")),
        };
        let pattern = if let Op::Like { prefix, suffix, .. } = op {
            format!(
                "{}{}{}",
                if prefix { "%" } else { "" },
                p,
                if suffix { "%" } else { "" }
            )
        } else {
            p
        };
        out.push(Cond::Cmp {
            col,
            op,
            vals,
            pat: pattern,
        });
    }
    Ok(out)
}

/// SQL three-valued evaluation: None is NULL/unknown.
fn eval(st: &State, t: T, rid: u32, c: &Cond) -> Option<bool> {
    match c {
        Cond::And(cs) => {
            let mut unknown = false;
            for c in cs {
                match eval(st, t, rid, c) {
                    Some(false) => return Some(false),
                    None => unknown = true,
                    _ => {}
                }
            }
            (!unknown).then_some(true)
        }
        Cond::Or(cs) => {
            let mut unknown = false;
            for c in cs {
                match eval(st, t, rid, c) {
                    Some(true) => return Some(true),
                    None => unknown = true,
                    _ => {}
                }
            }
            (!unknown).then_some(false)
        }
        Cond::Cmp { col, op, vals, pat } => {
            let v = st.get(t, rid, *col);
            let cmp = |o: &V| v.cmp_sql(o);
            match op {
                Op::IsNull => Some(v.is_null()),
                Op::NotNull => Some(!v.is_null()),
                Op::Eq => v.eq_sql(&vals[0]),
                Op::Ne => v.eq_sql(&vals[0]).map(|b| !b),
                Op::Gt => cmp(&vals[0]).map(|o| o == Ordering::Greater),
                Op::Lt => cmp(&vals[0]).map(|o| o == Ordering::Less),
                Op::Gte => cmp(&vals[0]).map(|o| o != Ordering::Less),
                Op::Lte => cmp(&vals[0]).map(|o| o != Ordering::Greater),
                Op::In | Op::NotIn => {
                    if vals.is_empty() {
                        // drizzle renders inArray(col, []) as false and notInArray(col, []) as true
                        return Some(matches!(op, Op::NotIn));
                    }
                    if v.is_null() {
                        return None;
                    }
                    let mut unknown = false;
                    for x in vals {
                        match v.eq_sql(x) {
                            Some(true) => return Some(matches!(op, Op::In)),
                            None => unknown = true,
                            _ => {}
                        }
                    }
                    (!unknown).then_some(matches!(op, Op::NotIn))
                }
                Op::Like { nocase, negate, .. } => match &v {
                    V::Null => None,
                    V::Text(s) => Some(like(s, pat, *nocase) != *negate),
                    other => Some(like(&other_text(other), pat, *nocase) != *negate),
                },
            }
        }
    }
}

fn other_text(v: &V) -> String {
    match v {
        V::Int(i) => i.to_string(),
        V::Big(b) => b.to_string(),
        V::Text(s) => s.clone(),
        V::Null => String::new(),
    }
}

fn matches(st: &State, t: T, rid: u32, conds: &[Cond]) -> bool {
    conds.iter().all(|c| eval(st, t, rid, c) == Some(true))
}

/// Picks an index for a top-level `col == v` / `col_in [...]` condition, if any.
fn candidates(st: &State, t: T, def: &TableDef, conds: &[Cond]) -> Option<Vec<u32>> {
    let mut best: Option<Vec<u32>> = None;
    for c in conds {
        let Cond::Cmp { col, op, vals, .. } = c else {
            continue;
        };
        let name = def.cols[*col].name;
        let rows = match op {
            Op::Eq => st.index_lookup(t, name, &vals[0]),
            Op::In => {
                let mut all = Vec::new();
                let mut ok = true;
                for v in vals {
                    match st.index_lookup(t, name, v) {
                        Some(r) => all.extend(r),
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                ok.then(|| {
                    all.sort_unstable();
                    all.dedup();
                    all
                })
            }
            _ => None,
        };
        if let Some(r) = rows {
            if best.as_ref().is_none_or(|b| r.len() < b.len()) {
                best = Some(r);
            }
        }
    }
    best
}

fn order_schema(def: &TableDef, args: &PluralArgs) -> Result<Vec<(usize, bool)>, String> {
    let asc = args.order_direction.as_deref().unwrap_or("asc") == "asc";
    let mut cols = Vec::new();
    if let Some(ob) = &args.order_by {
        let c = def
            .col(ob)
            .ok_or_else(|| format!("Unknown column \"{ob}\" used in orderBy argument"))?;
        cols.push((c, asc));
    }
    for &pk in &def.pk {
        if !cols.iter().any(|(c, _)| *c == pk) {
            cols.push((pk, asc));
        }
    }
    Ok(cols)
}

pub fn encode_cursor(st: &State, def: &TableDef, schema: &[(usize, bool)], rid: u32) -> String {
    let mut json = serde_json::Map::new();
    let mut meta = serde_json::Map::new();
    for &(c, _) in schema {
        let name = def.cols[c].name.to_string();
        let v = match st.get(def.t, rid, c) {
            V::Null => serde_json::Value::Null,
            V::Int(i) => serde_json::Value::from(i),
            V::Big(b) => {
                meta.insert(name.clone(), serde_json::json!(["bigint"]));
                serde_json::Value::String(b.to_string())
            }
            V::Text(s) => serde_json::Value::String(s),
        };
        json.insert(name, v);
    }
    let mut obj = serde_json::Map::new();
    obj.insert("json".into(), serde_json::Value::Object(json));
    if !meta.is_empty() {
        obj.insert("meta".into(), serde_json::json!({ "values": meta }));
    }
    base64::engine::general_purpose::STANDARD.encode(serde_json::Value::Object(obj).to_string())
}

fn decode_cursor(def: &TableDef, schema: &[(usize, bool)], cursor: &str) -> Result<Vec<V>, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cursor.trim())
        .map_err(|_| "Invalid cursor".to_string())?;
    let j: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid cursor".to_string())?;
    schema
        .iter()
        .map(|&(c, _)| {
            let col = &def.cols[c];
            Ok(match (&j["json"][col.name], col.ty) {
                (serde_json::Value::Null, _) => V::Null,
                (serde_json::Value::Number(n), ColType::Int) => {
                    V::Int(n.as_i64().unwrap_or_default())
                }
                (serde_json::Value::Number(n), ColType::BigInt) => {
                    V::big_from_i64(n.as_i64().unwrap_or_default())
                }
                (serde_json::Value::String(s), ColType::BigInt) => {
                    V::parse_big(s).ok_or_else(|| "Invalid cursor".to_string())?
                }
                (serde_json::Value::String(s), ColType::Int) => {
                    V::Int(s.parse().map_err(|_| "Invalid cursor".to_string())?)
                }
                (serde_json::Value::String(s), ColType::Text) => V::Text(s.clone()),
                (other, _) => V::Text(other.to_string()),
            })
        })
        .collect()
}

/// buildCursorCondition: (c1 > v1) OR (c1 = v1 AND (... cn >= vn)), with SQL NULL semantics.
fn cursor_ok(
    st: &State,
    t: T,
    rid: u32,
    schema: &[(usize, bool)],
    vals: &[V],
    after: bool,
) -> bool {
    fn go(
        st: &State,
        t: T,
        rid: u32,
        schema: &[(usize, bool)],
        vals: &[V],
        after: bool,
        i: usize,
    ) -> Option<bool> {
        let (c, asc) = schema[i];
        let v = st.get(t, rid, c);
        let o = v.cmp_sql(&vals[i]);
        let forward = after == asc; // true: compare with >, false: with <
        let strict = o.map(|o| {
            if forward {
                o == Ordering::Greater
            } else {
                o == Ordering::Less
            }
        });
        if i == schema.len() - 1 {
            return o.map(|o| {
                if forward {
                    o != Ordering::Less
                } else {
                    o != Ordering::Greater
                }
            });
        }
        let eq = o.map(|o| o == Ordering::Equal);
        let rest = go(st, t, rid, schema, vals, after, i + 1);
        let and = match (eq, rest) {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), Some(true)) => Some(true),
            _ => None,
        };
        match (strict, and) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (Some(false), Some(false)) => Some(false),
            _ => None,
        }
    }
    go(st, t, rid, schema, vals, after, 0) == Some(true)
}

fn sort_rows(
    st: &State,
    t: T,
    schema: &[(usize, bool)],
    rows: &mut Vec<u32>,
    reverse: bool,
    keep: usize,
) {
    if schema.len() == 1 && State::natural_pk_order(t) && st_pk_is_rid(t) {
        // Slot order is primary key order.
        let asc = schema[0].1 != reverse;
        if asc {
            rows.sort_unstable();
        } else {
            rows.sort_unstable_by(|a, b| b.cmp(a));
        }
        rows.truncate(keep);
        return;
    }
    let mut keyed: Vec<(Vec<V>, u32)> = rows
        .iter()
        .map(|&r| (schema.iter().map(|&(c, _)| st.get(t, r, c)).collect(), r))
        .collect();
    let cmp = |a: &(Vec<V>, u32), b: &(Vec<V>, u32)| {
        for (i, &(_, asc)) in schema.iter().enumerate() {
            let o = a.0[i].cmp_order(&b.0[i], asc != reverse);
            if o != Ordering::Equal {
                return o;
            }
        }
        Ordering::Equal
    };
    if keep < keyed.len() {
        keyed.select_nth_unstable_by(keep, cmp);
        keyed.truncate(keep);
    }
    keyed.sort_unstable_by(cmp);
    *rows = keyed.into_iter().map(|k| k.1).collect();
}

fn st_pk_is_rid(t: T) -> bool {
    State::natural_pk_order(t)
}

/// Index lookups larger than this are slower than walking an ordered source.
const SMALL_CANDIDATES: usize = 20_000;

/// The first `keep` rows matching `conds` (and the cursor), in schema order (reversed if asked).
fn select(
    st: &State,
    def: &TableDef,
    schema: &[(usize, bool)],
    conds: &[Cond],
    cursor: Option<(&[V], bool)>,
    reverse: bool,
    keep: usize,
) -> Vec<u32> {
    let t = def.t;
    let ok = |r: u32| {
        matches(st, t, r, conds)
            && cursor.is_none_or(|(vals, after)| cursor_ok(st, t, r, schema, vals, after))
    };
    let cand = candidates(st, t, def, conds);
    if let Some(c) = &cand {
        if c.len() <= SMALL_CANDIDATES {
            let mut rows: Vec<u32> = c.iter().copied().filter(|&r| ok(r)).collect();
            sort_rows(st, t, schema, &mut rows, reverse, keep);
            return rows;
        }
    }

    // An ordered source: row ids in order of the first sort column.
    let (c0, asc0) = schema[0];
    let schema_is_pk =
        schema.len() == def.pk.len() && schema.iter().zip(&def.pk).all(|(s, p)| s.0 == *p);
    let perm: Option<&Vec<u32>> = if State::natural_pk_order(t) && c0 == 0 {
        None
    } else if State::monotonic(t, c0) {
        None
    } else if schema_is_pk
        && State::TEXT_PK_TABLES.contains(&t)
        && st.pk_order.get(&t).is_some_and(|o| o.len() == st.slots(t))
    {
        st.pk_order.get(&t)
    } else {
        let mut rows: Vec<u32> = match cand {
            Some(c) => c.into_iter().filter(|&r| ok(r)).collect(),
            None => (0..st.slots(t) as u32)
                .filter(|&r| st.exists(t, r) && ok(r))
                .collect(),
        };
        sort_rows(st, t, schema, &mut rows, reverse, keep);
        return rows;
    };
    let len = perm.map_or(st.slots(t), |p| p.len());
    let at = |i: usize| perm.map_or(i as u32, |p| p[i]);
    let grouped = perm.is_none() && !(State::natural_pk_order(t) && c0 == 0) && schema.len() > 1;
    let ascending = asc0 != reverse;

    // Skip straight to the cursor position.
    let (mut lo, mut hi) = (0usize, len);
    if let Some((vals, _)) = cursor {
        let c = &vals[0];
        // Brush, Canvas, Animation and Global use their id as the row id and may have gaps.
        let id_is_rid = perm.is_none()
            && matches!(t, T::Brush | T::Canvas | T::Animation | T::Global)
            && c0 == 0;
        let first = |i: usize| {
            if id_is_rid {
                V::Int(i as i64)
            } else {
                st.get(t, at(i), c0)
            }
        };
        let below = |i: usize, or_eq: bool| match first(i).cmp_sql(c) {
            Some(std::cmp::Ordering::Less) => true,
            Some(std::cmp::Ordering::Equal) => or_eq,
            _ => false,
        };
        if ascending {
            lo = partition(len, |i| below(i, false));
        } else {
            hi = partition(len, |i| below(i, true));
        }
    }

    let mut out = Vec::new();
    let mut run: Option<V> = None;
    let mut visit = |i: usize| -> bool {
        let r = at(i);
        if !st.exists(t, r) || !ok(r) {
            return true;
        }
        if grouped {
            let v = st.get(t, r, c0);
            if out.len() >= keep && run.as_ref() != Some(&v) {
                return false;
            }
            run = Some(v);
        } else if out.len() >= keep {
            return false;
        }
        out.push(r);
        true
    };
    if ascending {
        for i in lo..hi {
            if !visit(i) {
                break;
            }
        }
    } else {
        for i in (lo..hi).rev() {
            if !visit(i) {
                break;
            }
        }
    }
    if grouped {
        sort_rows(st, t, schema, &mut out, reverse, keep);
    }
    out.truncate(keep);
    out
}

/// First index in 0..len where `below` is false (`below` must be monotonic).
fn partition(len: usize, below: impl Fn(usize) -> bool) -> usize {
    let (mut lo, mut hi) = (0, len);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if below(mid) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

pub fn plural(
    st: &State,
    def: &TableDef,
    args: &PluralArgs,
    want_total: bool,
    extra: Option<(usize, V)>,
) -> Result<Page, String> {
    let t = def.t;
    let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = args.offset.unwrap_or(0).max(0);
    if limit > MAX_LIMIT {
        return Err(format!(
            "Invalid limit. Got {limit}, expected <={MAX_LIMIT}."
        ));
    }
    let limit = limit.max(0) as usize;
    let schema = order_schema(def, args)?;
    let mut conds = match &args.where_ {
        Some(w) => compile_where(def, w)?,
        None => vec![],
    };
    if let Some((col, v)) = extra {
        conds.push(Cond::Cmp {
            col,
            op: Op::Eq,
            vals: vec![v],
            pat: String::new(),
        });
    }
    if args.after.is_some() && args.before.is_some() {
        return Err("Cannot specify both before and after cursors.".into());
    }
    if args.after.is_some() && offset > 0 {
        return Err("Cannot specify both after cursor and offset.".into());
    }
    if args.before.is_some() && offset > 0 {
        return Err("Cannot specify both before cursor and offset.".into());
    }

    let total = want_total.then(|| match candidates(st, t, def, &conds) {
        Some(c) => c.into_iter().filter(|&r| matches(st, t, r, &conds)).count() as i64,
        None => (0..st.slots(t) as u32)
            .filter(|&r| st.exists(t, r) && matches(st, t, r, &conds))
            .count() as i64,
    });

    let mut page = Page {
        table: t,
        rows: vec![],
        total,
        has_next: false,
        has_prev: false,
        start_cursor: None,
        end_cursor: None,
    };
    let cursor = |p: &Page, i: usize| encode_cursor(st, def, &schema, p.rows[i]);

    match (&args.after, &args.before) {
        (None, None) => {
            let rows = select(
                st,
                def,
                &schema,
                &conds,
                None,
                false,
                offset as usize + limit + 1,
            );
            let mut rows: Vec<u32> = rows.into_iter().skip(offset as usize).collect();
            if rows.len() == limit + 1 {
                rows.pop();
                page.has_next = true;
            }
            page.rows = rows;
            if offset == 0 {
                if !page.rows.is_empty() {
                    page.start_cursor = Some(cursor(&page, 0));
                    page.end_cursor = Some(cursor(&page, page.rows.len() - 1));
                }
            } else {
                page.has_prev = true;
            }
        }
        (Some(after), None) => {
            let vals = decode_cursor(def, &schema, after)?;
            let rows = select(
                st,
                def,
                &schema,
                &conds,
                Some((&vals, true)),
                false,
                limit + 2,
            );
            if rows.is_empty() {
                return Ok(page);
            }
            page.rows = rows;
            if cursor(&page, 0) == *after {
                page.rows.remove(0);
                page.has_prev = true;
            } else {
                page.rows.pop();
            }
            if page.rows.len() == limit + 1 {
                page.rows.pop();
                page.has_next = true;
            }
            if !page.rows.is_empty() {
                page.start_cursor = Some(cursor(&page, 0));
                page.end_cursor = Some(cursor(&page, page.rows.len() - 1));
            }
        }
        (None, Some(before)) => {
            let vals = decode_cursor(def, &schema, before)?;
            let mut rows = select(
                st,
                def,
                &schema,
                &conds,
                Some((&vals, false)),
                true,
                limit + 2,
            );
            rows.reverse();
            if rows.is_empty() {
                return Ok(page);
            }
            page.rows = rows;
            if cursor(&page, page.rows.len() - 1) == *before {
                page.rows.pop();
                page.has_next = true;
            } else {
                page.rows.remove(0);
            }
            if page.rows.len() == limit + 1 {
                page.rows.remove(0);
                page.has_prev = true;
            }
            if !page.rows.is_empty() {
                page.start_cursor = Some(cursor(&page, 0));
                page.end_cursor = Some(cursor(&page, page.rows.len() - 1));
            }
        }
        _ => unreachable!(),
    }
    Ok(page)
}

/// Verifies that `pk_order` agrees with the generic collation comparison (CHECK_PK_ORDER=1).
pub fn check_pk_order(st: &State, defs: &[TableDef]) -> Result<(), String> {
    for def in defs.iter().filter(|d| State::TEXT_PK_TABLES.contains(&d.t)) {
        let order = &st.pk_order[&def.t];
        let key = |r: u32| {
            def.pk
                .iter()
                .map(|&c| st.get(def.t, r, c))
                .collect::<Vec<_>>()
        };
        for w in order.windows(2) {
            let (a, b) = (key(w[0]), key(w[1]));
            let o = a
                .iter()
                .zip(&b)
                .map(|(x, y)| x.cmp_order(y, true))
                .find(|o| *o != Ordering::Equal)
                .unwrap_or(Ordering::Equal);
            if o != Ordering::Less {
                return Err(format!("{}: {:?} sorted before {:?}", def.name, a, b));
            }
        }
        eprintln!("pk order ok: {} ({} rows)", def.name, order.len());
    }
    Ok(())
}
