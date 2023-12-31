# Intro

這是一個使用Axum框架進行開發的練習用網頁後端專案，專案目標是完成一個類似於登記系統的服務。

## 使用方式

1. cargo run：執行服務
2. cargo watch -q -c -w tests/ -x "test -q quick_dev -- --nocapture"：測試服務

## 學習目標

- [x] 了解如何使用Axum框架
- [x] 閱讀第一部分的影片並完成專案建置
- [x] 回顧專案並加上專案開發筆記，來紀錄自己不了解的地方跟學習到的地方
  - [x] main.rs
  - [x] model.rs
  - [x] log.rs
  - [x] error.rs
  - [x] ctx.rs
  - [x] web/mod.rs
  - [x] web/mw_auth.rs
  - [x] web/routes_login.rs
  - [x] web/routes_tickets.rs
- [ ] 閱讀第二部分的影片並完善專案（目前作者尚未發布）

## 參考來源

[Rust Axum Full Course - Web Development](https://www.youtube.com/watch?v=XZtlD_m59sM)
