use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DbErr, EntityTrait,
    Iden, IntoActiveModel, Iterable, PaginatorTrait, Selector, Value,
};

#[derive(Debug)]
pub struct UserInfo {
    id: String,
}

#[derive(Debug)]
pub struct HandlerContext {
    user: Option<UserInfo>,
}

impl HandlerContext {
    pub fn new(user: Option<UserInfo>) -> Self {
        Self { user }
    }

    pub fn authenticated_user(&self) -> Option<&UserInfo> {
        self.user.as_ref()
    }
}

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::persistence::entities::user;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct ReqIdDto {
    pub id: i64,
}

#[derive(Serialize, Debug)]
pub struct ReqPaginationDto {
    pub page: Option<u64>,
    pub items_per_page: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct ResultPagination {
    pub current_page: u64,
    pub items_per_page: u64,
    pub total_items: u64,
    pub total_pages: u64,
}

#[derive(Serialize, Debug)]
pub struct ResFilterResultDto<T>
where
    T: Serialize,
{
    pub pagination: ResultPagination,
    pub items: Option<Vec<T>>,
}

pub trait Creatable: ActiveModelTrait {
    fn fill_create_audit(&mut self, user_id: String);
}

pub trait Updatable: ActiveModelTrait {
    fn fill_update_audit(&mut self, user_id: String);
}

pub trait Deletable: ActiveModelTrait {
    fn fill_delete_audit(&mut self, user_id: String);
}

#[macro_export]
macro_rules! make_creatable {
    ($model:ty) => {
        impl Creatable for $model {
            fn fill_create_audit(&mut self, user_id: String) {
                let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
                self.created_at = Set(now);
                self.created_by = Set(user_id.clone());

                for col in <<$model as ActiveModelTrait>::Entity as EntityTrait>::Column::iter() {
                    match col.to_string().as_str() {
                        "updated_at" => {
                            self.set(col, Value::from(Some(now)));
                        }
                        "updated_by" => {
                            self.set(col, Value::from(Some(user_id.clone())));
                        }
                        _ => {}
                    }
                }
            }
        }
    };
}

make_creatable!(user::ActiveModel);

#[macro_export]
macro_rules! make_updatable {
    ($model:ty) => {
        impl Updatable for $model {
            fn fill_update_audit(&mut self, user_id: String) {
                self.updated_at = Set(Some(chrono::Utc::now().into()));
                self.updated_by = Set(Some(user_id));
            }
        }
    };
}

make_updatable!(user::ActiveModel);

#[macro_export]
macro_rules! make_deletable {
    ($model:ty) => {
        impl $Deletable for $model {
            fn fill_delete_audit(&mut self, user_id: i64) {
                self.deleted_at = Set(Some(chrono::Utc::now().into()));
                self.deleted_by = Set(Some(user_id));
            }
        }
    };
}

pub struct Repository<'a, C: ConnectionTrait> {
    pub db: &'a C,
}

impl<'a, C: ConnectionTrait> Repository<'a, C> {
    pub fn new(db: &'a C) -> Self {
        Self { db }
    }

    pub async fn create<E>(
        &self,
        context: &HandlerContext,
        mut entity: E,
    ) -> Result<<E::Entity as EntityTrait>::Model, DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Creatable + ActiveModelBehavior + Send + 'a,
    {
        let Some(user) = context.authenticated_user() else {
            return Err(DbErr::Custom("user required for this operation".to_owned()));
        };

        entity.fill_create_audit(user.id.clone());

        entity.insert(self.db).await
    }

    pub async fn delete<E>(
        &self,
        context: &HandlerContext,
        // Use whatever primary key value type Entity E requires
        id: <<E::Entity as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType,
    ) -> Result<<E::Entity as EntityTrait>::Model, DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Deletable + ActiveModelBehavior + Send + 'a,
    {
        let Some(user) = context.authenticated_user() else {
            return Err(DbErr::Custom("user required for this operation".to_owned()));
        };
        let model = E::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Record not found".to_owned()))?;
        let mut model = model.into_active_model();

        model.fill_delete_audit(user.id.clone());

        model.update(self.db).await
    }

    pub async fn update<E>(
        &self,
        context: &HandlerContext,
        // Use whatever primary key value type Entity E requires
        id: <<E::Entity as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType,
        fill_values: impl FnOnce(&mut E),
    ) -> Result<<E::Entity as EntityTrait>::Model, DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Updatable + ActiveModelBehavior + Send + 'a,
    {
        let Some(user) = context.authenticated_user() else {
            return Err(DbErr::Custom("user required for this operation".to_owned()));
        };
        let model = E::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Record not found".to_owned()))?;
        let mut model = model.into_active_model();

        fill_values(&mut model);
        model.fill_update_audit(user.id.clone());

        model.update(self.db).await
    }

    pub async fn paginate_query<M, T, F>(
        &self,
        selector: Selector<sea_orm::SelectModel<M>>,
        page: Option<u64>,
        items_per_page: Option<u64>,
        map_fn: F,
    ) -> Result<ResFilterResultDto<T>, DbErr>
    where
        M: sea_orm::FromQueryResult + Sized + Send + Sync,
        T: serde::Serialize,
        F: Fn(M) -> T,
    {
        let current_page = page.unwrap_or(1).max(1);
        let items_per_page = items_per_page.unwrap_or(10).clamp(1, 100);

        let paginator = selector.paginate(self.db, items_per_page);
        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        let result = paginator.fetch_page(current_page - 1).await?;

        let items = result.into_iter().map(map_fn).collect::<Vec<T>>();

        Ok(ResFilterResultDto {
            pagination: ResultPagination {
                current_page,
                items_per_page,
                total_items,
                total_pages,
            },
            items: Some(items),
        })
    }
}
