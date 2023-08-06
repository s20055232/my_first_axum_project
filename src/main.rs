#![allow(unused)]

use crate::{log::log_request, model::ModelController, web::mw_auth};

use self::error::{Error, Result};
use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router, ServiceExt,
};
use ctx::Ctx;
use serde::Deserialize;
use serde_json::json;
use std::{fmt::format, net::SocketAddr};
use tokio::signal;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod ctx;
mod error;
mod log;
mod model;
mod web;
#[tokio::main]
async fn main() -> Result<()> {
    // 先建立我們的資料庫
    let mc = ModelController::new().await?;
    // 我們ticket相關的API呼叫，需要經過權限認證，因此我們加上一層middleware來進行驗證的動作
    // 而因為我們只希望權限驗證發生在這邊，所以我們使用route_layer，而不是layer
    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(mw_auth::mw_require_auth));
    // fallback_service: 如果沒有匹配到任何Route，將會使用這邊提供的服務。
    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> Listen on addr: {addr}");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();
    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());
    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body =
                json!({"error": {"type": client_error.as_ref(), "req_uuid": uuid.to_string()}});
            println!("   ->> client_error_body: {client_error_body}");

            // Build the new response from client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;
    println!();
    error_response.unwrap_or(res)
}
fn routes_static() -> Router {
    // nest_service: 將提供的service包裝在指定的path之下
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(hello_handler))
        .route("/hello2/:name", get(handler_hello2))
}
#[derive(Debug, Deserialize)]
struct HelloParam {
    name: Option<String>,
}
async fn hello_handler(Query(param): Query<HelloParam>) -> impl IntoResponse {
    println!("->> {:<12} - handle_hello - {param:?}", "HANDLER");

    let name = param.name.as_deref().unwrap_or("World!!!");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handle_hello - {name:?}", "HANDLER");

    Html(format!("Hello <strong>{name}</strong>"))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
