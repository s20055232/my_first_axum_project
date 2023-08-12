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

    // 所有路由的匯總之處，透過merge可以將路由一部份一部份的加上去
    // 如果是一般的.route()則是添加一個路由handler
    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        // nest的作用是幫你把提供的路由再包上一層
        .nest("/api", routes_apis)
        // layer是全域範圍的，可以幫你對routes做額外的處理，要留意的是，layer會對你已存在的routes作處理，但不會處理後來添加的，
        // 所以使用上，你要先添加routes，再添加layer，且越後面添加的layer會越先執行
        // map_response會對你的response在做一次處理
        .layer(middleware::map_response(main_response_mapper))
        // middleware：在request進來時先做處理，並使用Application level的State作為參數
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        // fallback_service: 如果沒有匹配到任何Route，將會使用這邊提供的服務。
        .fallback_service(routes_static());

    // 綁定一個SocketAddr變數
    // 也可以透過此方式綁定：let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    // 但比較繁瑣，使用from可以直接進行轉換
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> Listen on addr: {addr}");
    axum::Server::bind(&addr)
        // serve參數提供的routes_all要呼叫.into_make_service才能使用
        // 因為serve這個方法是hyper所定義的，而.into_make_service會幫你將axum的Router轉換成hyper可以接受的格式
        .serve(routes_all.into_make_service())
        // 這邊加上教學中沒有的with_graceful_shutdown，這個可以為這個服務加上一個服務用來處理特定信號
        // 以我們這邊為例，當接受ctrl+c或是任何terminate的信號，就會觸發並在等候其他之前接受到的request完成之後將服務關閉
        // 這邊也可以實作一些關閉連接、清除資源等等的動作，但也可以透過實作連接的物件的drop來處理，端看怎麼設計
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

// 每個route在回傳前都會先經過此段邏輯，在這邊我們會將錯誤訊息做處理，避免內部錯誤訊息讓外部知道
async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();
    // -- Get the eventual response error.
    // 我們附加的資訊像是錯誤、State之類的可以存儲在extension裡面傳遞，相當方便
    let service_error = res.extensions().get::<Error>();
    // 將我們附加的錯誤訊息取出，並呼叫轉換，以避免內部訊息洩漏出去
    let client_status_error = service_error.map(|se| se.client_status_and_error());
    // -- If client error, build the new response.
    let error_response = client_status_error
        // 當我們想要借用別人的值，但我們不需要外部的嵌套(Option)時，可以使用as_ref
        .as_ref()
        // 並接續使用map，可以將內部數值直接轉換，不用使用到我們as_ref的對象的所有權
        .map(|(status_code, client_error)| {
            let client_error_body =
                // client_error可以呼叫as_ref是因為使用strum_macros::AsRefStr
                // 可以將被衍生的enum其中的值都轉換成'static str取代原本使用的String，減少浪費開銷
                json!({"error": {"type": client_error.as_ref(), "req_uuid": uuid.to_string()}});
            println!("   ->> client_error_body: {client_error_body}");

            // Build the new response from client_error_body
            // 針對(StatusCode, R)的tuple組合，axum有對此設計API提供使用，讓我們可以快速產生Response struct
            // 其中R的限制是該struct需要實作IntoResponse trait，而大多數axum提供struct都已經有實作，相當方便
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    // unzip 將一個Some((a, b))轉換成(Some(a), Some(b))，若沒有值，則(None, None)
    // 這樣使用可以讓我不需要先針對外層的Option先做操作再處理裡面的數值
    let client_error = client_status_error.unzip().1;
    // 進行log紀錄
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;
    println!();
    // 如果有錯誤，unwarp並回傳，不然就正常回傳結果
    error_response.unwrap_or(res)
}
fn routes_static() -> Router {
    // nest_service: 將提供的service包裝在指定的path之下
    // 這邊我們將檔案目錄下的內容直接提供外部存取，並包在"/"路徑底下，假使我們根目錄底下有1.png
    // 別人可以直接使用 https://<domain-name>/1.png來取得
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
fn routes_hello() -> Router {
    // 簡單的範例，定義了一個GET方法的API跟一個GET方法使用Query的API
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
