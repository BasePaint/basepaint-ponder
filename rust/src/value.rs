use crate::tables::ColType;
use ethnum::U256;
use std::cmp::Ordering;

/// A column value as Postgres would hold it.
#[derive(Clone, Debug, PartialEq)]
pub enum V {
    Null,
    Int(i64),
    Big(U256),
    Text(String),
}

impl V {
    pub fn is_null(&self) -> bool {
        matches!(self, V::Null)
    }

    /// SQL comparison: None when either side is NULL.
    pub fn cmp_sql(&self, other: &V) -> Option<Ordering> {
        match (self, other) {
            (V::Null, _) | (_, V::Null) => None,
            (V::Int(a), V::Int(b)) => Some(a.cmp(b)),
            (V::Big(a), V::Big(b)) => Some(a.cmp(b)),
            (V::Int(a), V::Big(b)) => Some(if *a < 0 {
                Ordering::Less
            } else {
                U256::from(*a as u64).cmp(b)
            }),
            (V::Big(a), V::Int(b)) => Some(if *b < 0 {
                Ordering::Greater
            } else {
                a.cmp(&U256::from(*b as u64))
            }),
            (V::Text(a), V::Text(b)) => Some(collate(a, b)),
            _ => None,
        }
    }

    /// Equality under SQL rules (deterministic collation: byte equality).
    pub fn eq_sql(&self, other: &V) -> Option<bool> {
        match (self, other) {
            (V::Null, _) | (_, V::Null) => None,
            (V::Text(a), V::Text(b)) => Some(a == b),
            _ => self.cmp_sql(other).map(|o| o == Ordering::Equal),
        }
    }

    /// ORDER BY comparison, Postgres defaults: NULLS LAST for ASC, NULLS FIRST for DESC.
    pub fn cmp_order(&self, other: &V, asc: bool) -> Ordering {
        let o = match (self, other) {
            (V::Null, V::Null) => Ordering::Equal,
            (V::Null, _) => Ordering::Greater,
            (_, V::Null) => Ordering::Less,
            _ => self.cmp_sql(other).unwrap_or(Ordering::Equal),
        };
        if asc { o } else { o.reverse() }
    }

    /// Coerce a GraphQL input value into a column value.
    pub fn from_gql(v: &async_graphql::Value, ty: ColType) -> Result<V, String> {
        use async_graphql::Value as G;
        Ok(match (v, ty) {
            (G::Null, _) => V::Null,
            (G::Number(n), ColType::Int) => V::Int(
                n.as_i64()
                    .or_else(|| n.as_f64().map(|f| f as i64))
                    .ok_or("invalid number")?,
            ),
            (G::Number(n), ColType::BigInt) => V::big_from_i64(n.as_i64().ok_or("invalid number")?),
            (G::String(s), ColType::BigInt) => {
                V::parse_big(s).ok_or_else(|| format!("Cannot convert {s} to a BigInt"))?
            }
            (G::String(s), ColType::Int) => {
                V::Int(s.parse().map_err(|_| format!("Invalid Int {s}"))?)
            }
            (G::String(s), ColType::Text) => V::Text(s.clone()),
            (G::Number(n), ColType::Text) => V::Text(n.to_string()),
            (G::Boolean(b), _) => V::Text(b.to_string()),
            (other, _) => return Err(format!("Unsupported filter value {other}")),
        })
    }

    pub fn big(x: i128) -> V {
        V::Big(U256::from(x.max(0) as u128))
    }

    pub fn big_from_i64(i: i64) -> V {
        if i < 0 {
            V::Int(i)
        } else {
            V::Big(U256::from(i as u64))
        }
    }

    /// BigInt literal; negative values (never stored) compare as plain integers.
    pub fn parse_big(s: &str) -> Option<V> {
        if s.starts_with('-') {
            return s.parse().ok().map(V::Int);
        }
        U256::from_str_radix(s, 10).ok().map(V::Big)
    }

    pub fn to_gql(&self) -> async_graphql::Value {
        use async_graphql::Value as G;
        match self {
            V::Null => G::Null,
            V::Int(i) => G::from(*i),
            V::Big(b) => G::String(b.to_string()),
            V::Text(s) => G::String(s.clone()),
        }
    }
}

/// Approximates the glibc en_US.UTF-8 collation the production Postgres uses:
/// punctuation and spaces are ignored and case is ignored at the first level,
/// then lowercase sorts before uppercase, then earlier punctuation sorts first.
pub fn collate(a: &str, b: &str) -> Ordering {
    fn alnum(s: &str) -> impl Iterator<Item = char> + '_ {
        s.chars().filter(|c| c.is_alphanumeric())
    }
    fn punct(s: &str) -> impl Iterator<Item = (usize, char)> + '_ {
        s.char_indices().filter(|(_, c)| !c.is_alphanumeric())
    }
    alnum(a)
        .flat_map(char::to_lowercase)
        .cmp(alnum(b).flat_map(char::to_lowercase))
        .then_with(|| {
            alnum(a)
                .map(char::is_uppercase)
                .cmp(alnum(b).map(char::is_uppercase))
        })
        .then_with(|| punct(a).cmp(punct(b)))
        .then_with(|| a.as_bytes().cmp(b.as_bytes()))
}

/// SQL LIKE / ILIKE with `%`, `_` and backslash escapes.
pub fn like(s: &str, pat: &str, nocase: bool) -> bool {
    let (s, pat): (Vec<char>, Vec<char>) = if nocase {
        (
            s.chars().flat_map(|c| c.to_lowercase()).collect(),
            pat.chars().flat_map(|c| c.to_lowercase()).collect(),
        )
    } else {
        (s.chars().collect(), pat.chars().collect())
    };
    fn go(s: &[char], p: &[char]) -> bool {
        let (mut si, mut pi) = (0, 0);
        let (mut star_p, mut star_s) = (usize::MAX, 0);
        while si < s.len() {
            if pi < p.len() && p[pi] == '%' {
                star_p = pi;
                star_s = si;
                pi += 1;
                continue;
            }
            if pi < p.len() {
                let (pc, adv, any) = if p[pi] == '\\' && pi + 1 < p.len() {
                    (p[pi + 1], 2, false)
                } else {
                    (p[pi], 1, p[pi] == '_')
                };
                if any || pc == s[si] {
                    si += 1;
                    pi += adv;
                    continue;
                }
            }
            if star_p != usize::MAX {
                pi = star_p + 1;
                star_s += 1;
                si = star_s;
                continue;
            }
            return false;
        }
        while pi < p.len() && p[pi] == '%' {
            pi += 1;
        }
        pi == p.len()
    }
    go(&s, &pat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collation_matches_production_samples() {
        // Observed on graphql.basepaint.xyz: case-insensitive, underscores ignored.
        let mut ids = vec!["0x0A1EB3", "0x0a1DC3", "0x0a2Ffc", "0x0A1edA"];
        ids.sort_by(|a, b| collate(a, b));
        assert_eq!(ids, vec!["0x0a1DC3", "0x0A1EB3", "0x0A1edA", "0x0a2Ffc"]);
        let mut ids = vec!["100_5", "1000_1", "1001_2", "100_2"];
        ids.sort_by(|a, b| collate(a, b));
        assert_eq!(ids, vec!["1000_1", "1001_2", "100_2", "100_5"]);
        let mut ids = vec!["271_246", "27_1246", "271_2458"];
        ids.sort_by(|a, b| collate(a, b));
        assert_eq!(ids, vec!["271_2458", "27_1246", "271_246"]);
    }

    #[test]
    fn like_patterns() {
        assert!(like("0xAbC", "0x%", false));
        assert!(!like("0xAbC", "0xa%", false));
        assert!(like("0xAbC", "0xa%", true));
        assert!(like("10_5", "10\\_%", false));
        assert!(like("1005", "10_5", false));
        assert!(like("abc", "%b%", false));
        assert!(!like("abc", "%d%", false));
    }
}
