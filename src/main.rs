use std::net::Ipv6Addr;

use askama::Template;
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower::layer::layer_fn;
use tower_http::trace::TraceLayer;

mod spec;
mod util;

#[tokio::main]
async fn main() {
    println!("Printing something as early as possible so fly.io sees it.");

    // Filter traces based on the RUST_LOG env var, or, if it's not set,
    // default to show INFO-level details.
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        "axum=info,protocol_z_cash=info,tower_http=info,tracing=info".to_owned()
    });

    // Set up tracing
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let app = Router::new()
        .route("/", get(index))
        .route("/:identifier", get(identifier_page))
        .layer(layer_fn(util::RedirectDomain::new))
        .layer(TraceLayer::new_for_http());

    // IPv6 + IPv6 any addr
    let host = Ipv6Addr::UNSPECIFIED;
    let port = 8080;
    tracing::debug!("Listening on {}:{}", host, port);
    let listener = TcpListener::bind((host, port)).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> IndexTemplate {
    IndexTemplate {}
}

#[derive(Deserialize)]
struct IdentifierQuery {
    #[serde(default, deserialize_with = "util::empty_string_as_true")]
    partial: bool,
}

#[derive(Template)]
#[template(path = "identifier.html")]
struct IdentifierTemplate {
    identifier: String,
    partial: bool,
    spec_source: String,
}

async fn identifier_page(
    Path(identifier): Path<String>,
    Query(params): Query<IdentifierQuery>,
) -> IdentifierTemplate {
    let spec_source = spec::get_location(&identifier);

    IdentifierTemplate {
        identifier,
        partial: params.partial,
        spec_source,
    }
}
