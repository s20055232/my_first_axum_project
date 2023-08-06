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
// Property Accessors. 限定外部只能使用我們提供的API來更改內部的值
impl Ctx {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}
