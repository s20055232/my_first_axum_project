use std::fmt::{write, Display};

use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

// server error，給內部除錯使用的訊息，會定義的更加清楚跟具體，並加上除錯所需的資訊，以方便排除錯誤
#[derive(Debug, Clone, strum_macros::AsRefStr, Serialize)]
// 可以將序列化的資料做轉換，如果今天錯誤是觸發TicketDeleteFailIdNotFound，enum 的 variant 會是 type 的值，而 id 會是 data 的值
// 則資料會被序列化為{"tag": "TicketDeleteFailIdNotFound", "data": "123"}
#[serde(tag = "type", content = "data")]
pub enum Error {
    LoginFail,
    // -- Auth errors.
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailCtxNotInRequestExt,
    // -- Model errors.
    TicketDeleteFailIdNotFound { id: u64 },
}

// 為我們自定義的Error實作標準庫Error的trait，要滿足條件需要實作Display跟Debug的trait
impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// 呼叫Statuscode的into_response會將Statuscode塞到response物件的status底下，並回傳一個response物件
// 我們再將自己實作的Error放進response的extension底下，這樣就有一個Response物件，裡面包含我們所要傳遞的錯誤訊息
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");
        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response
        response.extensions_mut().insert(self);

        response
    }
}

impl Error {
    // 將我們的內部錯誤代碼包裝，回傳HTTP StatusCode跟外部可看的代碼，才不會洩露資訊
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),
            // -- Auth
            Self::AuthFailCtxNotInRequestExt
            | Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),
            // -- Model
            Self::TicketDeleteFailIdNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // -- Fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

// client error，給外部看的，不會有太多細節，訊息也比較籠統
#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
}
