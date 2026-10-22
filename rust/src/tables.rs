// Table metadata mirroring ponder.schema.ts. The GraphQL schema is generated from this
// the same way Ponder generates it, so field order and names match the TypeScript indexer.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum T {
    Global,
    Account,
    Brush,
    Contribution,
    Usage,
    Stroke,
    Withdrawal,
    Canvas,
    Animation,
    Balance,
    TotalBalance,
}

pub const ALL: [T; 11] = [
    T::Global,
    T::Account,
    T::Brush,
    T::Contribution,
    T::Usage,
    T::Stroke,
    T::Withdrawal,
    T::Canvas,
    T::Animation,
    T::Balance,
    T::TotalBalance,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColType {
    Int,
    BigInt,
    Text,
}

pub struct Col {
    pub name: &'static str,
    pub ty: ColType,
    pub not_null: bool,
}

pub enum Rel {
    /// `local` column of this table references the primary key of `table`.
    One {
        name: &'static str,
        table: T,
        local: &'static str,
    },
    /// Rows of `table` whose `remote` column equals this row's `local` column.
    Many {
        name: &'static str,
        table: T,
        local: &'static str,
        remote: &'static str,
    },
}

pub struct TableDef {
    pub t: T,
    pub name: &'static str,
    pub cols: Vec<Col>,
    pub pk: Vec<usize>,
    pub rels: Vec<Rel>,
}

impl TableDef {
    pub fn col(&self, name: &str) -> Option<usize> {
        self.cols.iter().position(|c| c.name == name)
    }
}

fn c(name: &'static str, ty: ColType, not_null: bool) -> Col {
    Col { name, ty, not_null }
}

pub fn defs() -> Vec<TableDef> {
    use ColType::*;
    let one = |name, table, local, _remote: &'static str| Rel::One { name, table, local };
    let many = |name, table, local, remote| Rel::Many {
        name,
        table,
        local,
        remote,
    };
    vec![
        TableDef {
            t: T::Global,
            name: "Global",
            cols: vec![
                c("id", Int, true),
                c("startedAt", Int, true),
                c("epochDuration", Int, true),
                c("totalArtists", Int, true),
                c("totalPixels", Int, true),
                c("totalEarnings", BigInt, true),
                c("totalEarningsUsd8", BigInt, true),
                c("totalWithdrawals", BigInt, true),
                c("totalMints", Int, true),
                c("totalBurns", Int, true),
                c("totalSubscriptions", Int, true),
            ],
            pk: vec![0],
            rels: vec![],
        },
        TableDef {
            t: T::Account,
            name: "Account",
            cols: vec![
                c("id", Text, true),
                c("totalPixels", Int, true),
                c("totalWithdrawn", BigInt, true),
                c("totalEarned", BigInt, true),
                c("streak", Int, true),
                c("longestStreak", Int, true),
                c("lastPaintedDay", Int, false),
                c("totalDaysPainted", Int, true),
            ],
            pk: vec![0],
            rels: vec![
                many("brushes", T::Brush, "id", "ownerId"),
                many("contributions", T::Contribution, "id", "accountId"),
                many("strokes", T::Stroke, "id", "accountId"),
                many("withdrawals", T::Withdrawal, "id", "accountId"),
                many("balances", T::Balance, "id", "ownerId"),
                many("totalBalances", T::TotalBalance, "id", "ownerId"),
            ],
        },
        TableDef {
            t: T::Brush,
            name: "Brush",
            cols: vec![
                c("id", Int, true),
                c("ownerId", Text, true),
                c("strength", Int, true),
                c("lastUsedTimestamp", Int, false),
                c("lastUsedDay", Int, false),
                c("mintedTimestamp", Int, true),
                c("streak", Int, true),
            ],
            pk: vec![0],
            rels: vec![
                one("owner", T::Account, "ownerId", "id"),
                many("usages", T::Usage, "id", "brushId"),
                many("strokes", T::Stroke, "id", "brushId"),
            ],
        },
        TableDef {
            t: T::Contribution,
            name: "Contribution",
            cols: vec![
                c("id", Text, true),
                c("accountId", Text, true),
                c("canvasId", Int, true),
                c("pixelsCount", Int, true),
            ],
            pk: vec![2, 1],
            rels: vec![
                one("account", T::Account, "accountId", "id"),
                one("canvas", T::Canvas, "canvasId", "id"),
            ],
        },
        TableDef {
            t: T::Usage,
            name: "Usage",
            cols: vec![
                c("id", Text, true),
                c("brushId", Int, true),
                c("canvasId", Int, true),
                c("pixelsCount", Int, true),
            ],
            pk: vec![0],
            rels: vec![
                one("brush", T::Brush, "brushId", "id"),
                one("canvas", T::Canvas, "canvasId", "id"),
            ],
        },
        TableDef {
            t: T::Stroke,
            name: "Stroke",
            cols: vec![
                c("id", BigInt, true),
                c("canvasId", Int, true),
                c("accountId", Text, true),
                c("brushId", Int, true),
                c("data", Text, true),
                c("pixels", Int, true),
                c("tx", Text, true),
                c("timestamp", Int, true),
            ],
            pk: vec![0],
            rels: vec![
                one("canvas", T::Canvas, "canvasId", "id"),
                one("account", T::Account, "accountId", "id"),
                one("brush", T::Brush, "brushId", "id"),
            ],
        },
        TableDef {
            t: T::Withdrawal,
            name: "Withdrawal",
            cols: vec![
                c("id", Text, true),
                c("accountId", Text, true),
                c("canvasId", Int, true),
                c("amount", BigInt, true),
                c("timestamp", Int, true),
            ],
            pk: vec![0],
            rels: vec![
                one("account", T::Account, "accountId", "id"),
                one("canvas", T::Canvas, "canvasId", "id"),
            ],
        },
        TableDef {
            t: T::Canvas,
            name: "Canvas",
            cols: vec![
                c("id", Int, true),
                c("totalMints", Int, true),
                c("totalBurns", Int, true),
                c("totalEarned", BigInt, true),
                c("totalEarnedUsd8", BigInt, true),
                c("ethUsdPriceAtStart8", BigInt, true),
                c("totalArtists", Int, true),
                c("pixelsCount", Int, true),
                c("name", Text, false),
                c("palette", Text, false),
                c("size", Int, false),
                c("proposer", Text, false),
            ],
            pk: vec![0],
            rels: vec![
                many("contributions", T::Contribution, "id", "canvasId"),
                many("usages", T::Usage, "id", "canvasId"),
                many("strokes", T::Stroke, "id", "canvasId"),
                many("withdrawals", T::Withdrawal, "id", "canvasId"),
            ],
        },
        TableDef {
            t: T::Animation,
            name: "Animation",
            cols: vec![c("id", Int, true), c("totalMints", Int, true)],
            pk: vec![0],
            rels: vec![],
        },
        TableDef {
            t: T::Balance,
            name: "Balance",
            cols: vec![
                c("id", Text, true),
                c("ownerId", Text, true),
                c("contract", Text, true),
                c("tokenId", BigInt, true),
                c("value", Int, true),
            ],
            pk: vec![0],
            rels: vec![one("owner", T::Account, "ownerId", "id")],
        },
        TableDef {
            t: T::TotalBalance,
            name: "TotalBalance",
            cols: vec![
                c("id", Text, true),
                c("ownerId", Text, true),
                c("contract", Text, true),
                c("value", Int, true),
            ],
            pk: vec![0],
            rels: vec![one("owner", T::Account, "ownerId", "id")],
        },
    ]
}
