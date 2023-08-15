// 這段程式碼定義了一個名為Ctx的結構（struct），用於表示一種上下文（context）物件。
// 在很多應用程式中，上下文物件常用於保存請求或應用程式運行期間需要的資訊，如當前用戶的ID、設定參數、數據庫連接等。
// 我們可以透過將相似且經常一起使用的部分封裝在一起，來簡化參數的數量，並且可以設計API供外部使用，確保外部使用符合預期

#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: u64,
}

// Constructor.
impl Ctx {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }
}
// Property Accessors. 限定外部只能使用我們提供的API來取得內部的值
// 可以確保資料的安全性跟完整性
impl Ctx {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}
