#[derive(Debug, Clone)]
pub struct DbContext {
    pub id: String,
}

impl DbContext {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn system() -> Self {
        Self {
            id: "system".to_string(),
        }
    }
}
