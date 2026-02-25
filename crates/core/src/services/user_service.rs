use crate::domain::{
    entities::user::User,
    ports::user_repo::{UserError, UserRepository},
};
use ro_common::id::generate_nanoid;
use std::sync::Arc; // Reusing your shared lib

#[derive(Debug, Clone)]
pub struct UserService {
    // The service owns the Abstract Repository (Port), not the Concrete Adapter.
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn register_user(&self, username: String, email: String) -> Result<User, UserError> {
        // 1. Business Logic: Generate ID
        let id = generate_nanoid();

        // 2. Business Logic: Create Domain Entity
        // (Here you could add checks, e.g., validate email format)
        let new_user = User::new(id, username, email);

        // 3. Persistence: Call the Port
        self.repo.save(&new_user).await?;

        Ok(new_user)
    }

    pub async fn get_user(&self, id: &str) -> Result<User, UserError> {
        self.repo.find_by_id(id).await?.ok_or(UserError::NotFound)
    }
}
