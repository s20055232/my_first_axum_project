//! Simplistic Model Layer
//! (with mock-store layer)

// MVC架構下，有模型（Model）、視圖（View）、控制器（Controller）三層
// 模型層負責資料的定義與資料庫的互動，包含對資料的CRUD操作。
// 複雜的邏輯運算、資料處理大多會在這一層完成，controller僅作呼叫內部已經定義好的功能並回傳。
use crate::{ctx::Ctx, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
#[derive(Debug, Clone, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64, // creator user_id
    pub title: String,
}

#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}

// 一般來說是使用資料庫存取，這邊為了簡單，使用物件（記憶體）當作資料庫使用
#[derive(Clone)]
pub struct ModelController {
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

impl ModelController {
    // Rust中，self是參數，代表呼叫此方法的物件本身，而Self則是指此方法實作的那個型別
    // self小寫，指定的單位較小，是特定物件，Self大寫，指定的是型別，
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }
}

// CRUD Implementation
impl ModelController {
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();
        let id = store.len() as u64;
        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };
        store.push(Some(ticket.clone()));
        Ok(ticket)
    }
    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let store = self.tickets_store.lock().unwrap();
        // filter_map 只會將 Some 類別的篩選出來
        let tickets = store.iter().filter_map(|t| t.clone()).collect();
        Ok(tickets)
    }
    // 給予要刪除的id，並將該id從資料庫中刪除
    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();
        // 1. get_mut 裡面不能直接使用id，必須convert成usize，因為SliceIndex只有usize有實作
        // 2. Option的take方法是將Option裡面的數值取出，並留下None
        let ticket = store.get_mut(id as usize).and_then(|t| t.take());
        // ok_or 可以將 Some 轉成 Result，相當好用
        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}
