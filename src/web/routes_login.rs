use crate::{web::AUTH_TOKEN, Error, Result};
use axum::{
    routing::{post, Route},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

// 建立一個子藍圖，底下包括跟登入有關的部分
pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

// 登入，幫使用者加上cookie
async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");
    if payload.username != "demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
    }
    cookies.add(Cookie::new(AUTH_TOKEN, "user-1.exp.sign"));
    let body = Json(json!({"result": {"success": true}}));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
