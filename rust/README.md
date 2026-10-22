# basepaint-indexer (Rust prototype)

A single-binary replacement for the Ponder indexer and its GraphQL server. It keeps every
table in memory (no Postgres), indexes the same events with the same handlers as `src/*.ts`,
and serves a GraphQL API that matches `graphql.basepaint.xyz` field for field.

## Run

```sh
cargo build --release
PONDER_RPC_URLS_8453=https://base-mainnet.g.alchemy.com/v2/KEY ./target/release/basepaint-indexer
```

The first start backfills every log with `eth_getLogs` (about 1,900 calls, 3–4 minutes on
Alchemy) plus about 4,900 `eth_call`s (Chainlink price per day, brush strength), and caches both
in `DATA_DIR`. After that a restart replays the cache in about 7 seconds and keeps polling the head.

| Env | Default | |
| --- | --- | --- |
| `PONDER_RPC_URLS_8453` | `https://mainnet.base.org` | Comma-separated, tried in order |
| `PONDER_RPC_TIMEOUT_MS` | `10000` | Per request |
| `PONDER_POLLING_INTERVAL_MS` | `2000` | Head polling |
| `PORT` | `42069` | |
| `DATA_DIR` | `data` | `logs.bin` (~1.3 GB), `calls.tsv`, `synced_to` |
| `REORG_WINDOW_BLOCKS` | `64` | Tail re-checked on every poll |
| `NO_SYNC` | unset | Serve the cache without polling |
| `CHECK_PK_ORDER` | unset | Verify the text primary-key indexes at startup |

`--print-schema` prints the SDL and exits. Routes are the same as Ponder's: `/` and `/graphql`
(GraphiQL on GET), `/health`, `/ready`, `/status`.

## How it works

- `logs.rs`: compact binary log store, append-only, mmapped at startup.
- `state.rs`: the tables as plain structs plus secondary indexes, and the event handlers ported
  1:1 from `src/*.ts`.
- `query.rs`: Ponder's `where` / `orderBy` / cursor semantics. Large tables are walked in an
  ordered source (row id order, a monotonic column, or a sorted primary-key index) so paging
  stops early instead of sorting.
- `gql.rs`: the dynamic schema, generated the way Ponder generates it.
- `sync.rs`: polling and reorgs. Each poll re-reads the last `REORG_WINDOW_BLOCKS` blocks and
  compares log fingerprints; on a mismatch it truncates the store and rebuilds the state.

## Compatibility checks (against prod, 2026-10-22)

- Introspected schema: identical SDL.
- 69 GraphQL queries pulled from basepaint-app: 69/69 identical responses. Locally 1–54 ms each,
  versus 57–600 ms on prod.
- 40 cursor walks (`after`/`before`, mixed `orderBy`, filters): identical.
- `globals`, `accounts`, `brushs`, `canvass`, `animations`: every row identical.
- `contributions`, `usages`, `strokes`, `withdrawals`: identical counts, 25 random 100-row
  pages each identical.
- Memory: about 0.9 GB RSS with the log file mapped.

## Known differences

- Text ordering emulates Postgres `en_US` collation (case and punctuation are ignored at the
  first level). It matches on every id we have, but it is not a full ICU/glibc implementation.
- An explicit `orderBy: null` errors on prod and is accepted here.
- No graphql-armor limits (max tokens / aliases); query depth is capped at 100.
- Reorgs deeper than `REORG_WINDOW_BLOCKS` are not detected.
