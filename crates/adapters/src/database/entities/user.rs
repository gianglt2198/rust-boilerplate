use ro_core::domain::entities::user::User;
use sea_orm::{ActiveValue::Set, entity::prelude::*};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub created_by: String,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub updated_by: Option<String>,
}

// 2. Define Relationships (None for now)
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for User {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            active: value.active,
        }
    }
}

impl From<User> for ActiveModel {
    fn from(value: User) -> Self {
        Self {
            id: Set(value.id.clone()),
            username: Set(value.username.clone()),
            email: Set(value.email.clone()),
            active: Set(value.active),
            ..Default::default()
        }
    }
}
