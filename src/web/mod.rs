// 將這邊有引入的module視為同一個module
pub mod mw_auth;
pub mod routes_login;
pub mod routes_tickets;
// 定義module共用的常數
pub const AUTH_TOKEN: &str = "auth-token";
