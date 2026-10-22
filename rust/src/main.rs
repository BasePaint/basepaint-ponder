mod calls;
mod gql;
mod logs;
mod query;
mod rpc;
mod state;
mod sync;
mod tables;
mod value;

use axum::extract::{Query, State as AxState};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
struct App {
    db: Arc<gql::Db>,
    schema: async_graphql::dynamic::Schema,
}

#[derive(serde::Deserialize)]
struct GqlBody {
    query: Option<String>,
    variables: Option<serde_json::Value>,
    #[serde(rename = "operationName")]
    operation_name: Option<String>,
}

async fn execute(app: &App, body: GqlBody) -> Response {
    let Some(q) = body.query else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "errors": [{ "message": "Must provide query string." }] })),
        )
            .into_response();
    };
    let mut req = async_graphql::Request::new(q);
    if let Some(v) = body.variables {
        req = req.variables(async_graphql::Variables::from_json(v));
    }
    if let Some(op) = body.operation_name {
        req = req.operation_name(op);
    }
    Json(app.schema.execute(req).await).into_response()
}

async fn graphql_post(AxState(app): AxState<App>, Json(body): Json<GqlBody>) -> Response {
    execute(&app, body).await
}

async fn graphql_get(
    AxState(app): AxState<App>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    match params.get("query") {
        None => Html(GRAPHIQL).into_response(),
        Some(q) => {
            let variables = params
                .get("variables")
                .and_then(|v| serde_json::from_str(v).ok());
            execute(
                &app,
                GqlBody {
                    query: Some(q.clone()),
                    variables,
                    operation_name: params.get("operationName").cloned(),
                },
            )
            .await
        }
    }
}

const GRAPHIQL: &str = r#"<!doctype html><html><head><title>BasePaint GraphQL</title>
<link rel="stylesheet" href="https://unpkg.com/graphiql@3.7.1/graphiql.min.css" /></head>
<body style="margin:0"><div id="g" style="height:100vh"></div>
<script crossorigin src="https://unpkg.com/react@18.3.1/umd/react.production.min.js"></script>
<script crossorigin src="https://unpkg.com/react-dom@18.3.1/umd/react-dom.production.min.js"></script>
<script crossorigin src="https://unpkg.com/graphiql@3.7.1/graphiql.min.js"></script>
<script>ReactDOM.createRoot(document.getElementById('g')).render(React.createElement(GraphiQL,{fetcher:GraphiQL.createFetcher({url:location.pathname})}))</script>
</body></html>"#;

fn env(k: &str, default: &str) -> String {
    std::env::var(k).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let urls: Vec<String> = env("PONDER_RPC_URLS_8453", "https://mainnet.base.org")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let timeout = Duration::from_millis(env("PONDER_RPC_TIMEOUT_MS", "10000").parse()?);
    let poll = Duration::from_millis(env("PONDER_POLLING_INTERVAL_MS", "2000").parse()?);
    let port: u16 = env("PORT", "42069").parse()?;
    let dir = std::path::PathBuf::from(env("DATA_DIR", "data"));
    let reorg_window: u64 = env("REORG_WINDOW_BLOCKS", "64").parse()?;

    let rpc = rpc::Rpc::new(urls, timeout);
    let mut store = sync::Store::open(dir, reorg_window)?;
    let st = store.build(&rpc, &state::Topics::new()).await?;
    if std::env::var("CHECK_PK_ORDER").is_ok() {
        query::check_pk_order(&st, &tables::defs()).map_err(anyhow::Error::msg)?;
    }
    let db = Arc::new(gql::Db::new(st));
    let schema = gql::build(db.clone());

    if std::env::args().any(|a| a == "--print-schema") {
        println!("{}", schema.sdl());
        return Ok(());
    }
    if std::env::var("NO_SYNC").is_err() {
        tokio::spawn(sync::run(db.clone(), store, rpc, poll));
    } else {
        db.ready.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let app = App { db, schema };
    let router = Router::new()
        .route("/", get(graphql_get).post(graphql_post))
        .route("/graphql", get(graphql_get).post(graphql_post))
        .route("/health", get(|| async { StatusCode::OK }))
        .route(
            "/ready",
            get(|AxState(app): AxState<App>| async move {
                if app.db.ready.load(std::sync::atomic::Ordering::Relaxed) {
                    (StatusCode::OK, "").into_response()
                } else {
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Historical indexing is not complete.",
                    )
                        .into_response()
                }
            }),
        )
        .route(
            "/status",
            get(|AxState(app): AxState<App>| async move { Json(app.db.status()) }),
        )
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(app);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    eprintln!("listening on :{port}");
    axum::serve(listener, router).await?;
    Ok(())
}
