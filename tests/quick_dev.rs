#![allow(unused)]
use anyhow::Result;
use axum::response::IntoResponse;
use serde_json::json;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    // 建立連接
    let hc = httpc_test::new_client("http://localhost:8080")?;
    // 嘗試query parameter是否成功
    hc.do_get("/hello?name=allen").await?.print().await?;
    // 嘗試path query是否成功
    hc.do_get("/hello2/allen").await?.print().await?;
    // 嘗試fallback_service是否成功，是否將可以獲取目錄底下的資源
    hc.do_get("/src/main.rs").await?.print().await?;
    // 嘗試登入api是否成功
    let req_login = hc.do_post("/api/login", json!({"username": "demo1", "pwd": "welcome"}));
    req_login.await?.print().await?;
    // 嘗試錯誤的登入帳密是否會被成功擋下來
    let req_login = hc.do_post("/api/login", json!({"username": "demo2", "pwd": "welcome"}));
    req_login.await?.print().await?;
    // 再次嘗試，目的是查看在登入時添加的cookie在下次呼叫時存在
    hc.do_get("/hello2/allen").await?.print().await?;
    // 嘗試是否可以正常添加ticket至資料庫
    let req_create_ticket = hc.do_post("/api/tickets", json!({"title": "Ticket AAA"}));
    req_create_ticket.await?.print().await?;
    // 嘗試將添加的ticket刪除
    hc.do_delete("/api/tickets/1").await?.print().await?;
    // 檢查我們添加的ticket，是否有成功添加
    hc.do_get("/api/tickets").await?.print().await?;
    Ok(())
}
