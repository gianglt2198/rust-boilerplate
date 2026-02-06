use async_trait::async_trait;
use ro_core::domain::{
    entities::user::User as DomainUser,
    ports::user_repo::{UserError, UserRepository},
};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::persistence::entities::user::{
    self, ActiveModel as UserActiveModel, Entity as UserEntity,
};

#[derive(Debug)]
pub struct PUserRepository {
    db: DatabaseConnection,
}

impl PUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<DomainUser>, UserError> {
        // 1. Fetch from DB using SeaORM
        let result = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| UserError::System(e.to_string()))?;

        // 2. Map Database Model -> Domain Entity
        match result {
            Some(model) => Ok(Some(
                user::Model {
                    id: model.id,
                    username: model.username,
                    email: model.email,
                    active: model.active,
                    created_at: model.created_at,
                    created_by: model.created_by,
                    updated_at: model.updated_at,
                    updated_by: model.updated_by,
                }
                .into(),
            )),
            None => Ok(None),
        }
    }

    async fn save(&self, user: &DomainUser) -> Result<(), UserError> {
        // 1. Map Domain Entity -> SeaORM ActiveModel
        // let user_model = UserActiveModel {
        //     id: Set(user.id.clone()),
        //     username: Set(user.username.clone()),
        //     email: Set(user.email.clone()),
        //     active: Set(user.active),
        // };
        let user = user.clone();
        let user_model: UserActiveModel = user.into();

        // 2. Execute Insert/Update (Upsert equivalent)
        // Note: SeaORM 'insert' will fail on duplicate.
        // For upsert, you typically check existence or use `on_conflict` if supported.
        // Here is a simple Insert for demonstration:
        user::Entity::insert(user_model)
            .exec(&self.db)
            .await
            .map_err(|e| UserError::System(e.to_string()))?;

        Ok(())
    }
}
