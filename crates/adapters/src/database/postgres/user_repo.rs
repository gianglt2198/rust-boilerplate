use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use ro_core::domain::{
    entities::user::User as DomainUser,
    ports::user_repo::{UserError, UserRepository},
};
use ro_db::orm::repo::Repository;
use sea_orm::{ConnectionTrait, EntityTrait};

use crate::database::entities::user::{self, ActiveModel as UserActiveModel, Entity as UserEntity};

#[derive(Debug, Clone)]
pub struct PUserRepository<C>
where
    C: ConnectionTrait + Send + Sync + Debug,
{
    repo: Repository<C>,
}

impl<C> PUserRepository<C>
where
    C: ConnectionTrait + Send + Sync + Debug,
{
    pub fn new(db: Arc<C>) -> Self {
        Self {
            repo: Repository::new(db),
        }
    }
}

#[async_trait]
impl<C> UserRepository for PUserRepository<C>
where
    C: ConnectionTrait + Send + Sync + Debug + 'static,
{
    async fn find_by_id(&self, id: &str) -> Result<Option<DomainUser>, UserError> {
        // 1. Fetch from DB using SeaORM
        let result = UserEntity::find_by_id(id)
            .one(self.repo.db.as_ref())
            .await
            .map_err(|e| UserError::System(e.to_string()))?;

        // 2. Map Database Model -> Domain Entity
        match result {
            Some(model) => Ok(Some(model.into())),
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
            .exec(self.repo.db.as_ref())
            .await
            .map_err(|e| UserError::System(e.to_string()))?;

        Ok(())
    }
}
