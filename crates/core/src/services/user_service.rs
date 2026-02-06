use crate::domain::{
    entities::user::User,
    ports::{
        messaging::EventPublisher,
        user_repo::{UserError, UserRepository},
    },
};
use ro_common::id::generate_nanoid;
use std::sync::Arc; // Reusing your shared lib

#[derive(Debug, Clone)]
pub struct UserService {
    // The service owns the Abstract Repository (Port), not the Concrete Adapter.
    repo: Arc<dyn UserRepository>,
    publisher: Arc<dyn EventPublisher>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, publisher: Arc<dyn EventPublisher>) -> Self {
        Self { repo, publisher }
    }

    pub async fn register_user(&self, username: String, email: String) -> Result<User, UserError> {
        // 1. Business Logic: Generate ID
        let id = generate_nanoid();

        // 2. Business Logic: Create Domain Entity
        // (Here you could add checks, e.g., validate email format)
        let new_user = User::new(id, username, email);

        // 3. Persistence: Call the Port
        self.repo.save(&new_user).await?;

        // 2. Publish Event (Fire & Forget or Await)
        if let Err(e) = self.publisher.publish_user_created(&new_user).await {
            // In production, you might log this error but not fail the request
            eprintln!("Failed to publish event: {:?}", e);
        }

        Ok(new_user)
    }

    pub async fn get_user(&self, id: &str) -> Result<User, UserError> {
        self.repo.find_by_id(id).await?.ok_or(UserError::NotFound)
    }
}
