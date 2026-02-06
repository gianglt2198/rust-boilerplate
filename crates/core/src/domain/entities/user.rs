use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub active: bool,
}

impl User {
    pub fn new(id: String, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            active: true,
        }
    }
}
