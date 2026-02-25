use std::sync::Arc;

use sea_orm::{
    ActiveModelBehavior, ConnectionTrait, DbErr, DeleteResult, EntityTrait, IntoActiveModel,
    PaginatorTrait, Selector,
};

use crate::orm::{
    audit::{Creatable, Deletable, Updatable},
    context::DbContext,
    dto::{ResFilterResultDto, ResultPagination},
};

#[derive(Debug, Clone)]
pub struct Repository<C: ConnectionTrait> {
    pub db: Arc<C>,
}

impl<C: ConnectionTrait> Repository<C> {
    pub fn new(db: Arc<C>) -> Self {
        Self { db }
    }

    pub async fn create<E>(
        &self,
        ctx: &DbContext,
        mut entity: E,
    ) -> Result<<E::Entity as EntityTrait>::Model, DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Creatable + ActiveModelBehavior + Send,
    {
        entity.fill_create_audit(ctx.id.clone());
        entity.insert(self.db.as_ref()).await
    }

    pub async fn create_many<E>(&self, ctx: &DbContext, mut entities: Vec<E>) -> Result<(), DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Creatable + ActiveModelBehavior + Send,
    {
        for entity in &mut entities {
            entity.fill_create_audit(ctx.id.clone());
        }
        E::Entity::insert_many(entities)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    pub async fn delete<E>(
        &self,
        ctx: &DbContext,
        // Use whatever primary key value type Entity E requires
        id: <<E::Entity as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType,
    ) -> Result<DeleteResult, DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Deletable + ActiveModelBehavior + Send,
    {
        let model = E::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(DbErr::RecordNotFound("Record not found".to_owned()))?;
        let mut model = model.into_active_model();
        if model.should_be_soft() {
            model.fill_delete_audit(ctx.id.clone());

            return match model.update(self.db.as_ref()).await {
                Ok(_) => Ok(DeleteResult { rows_affected: 1 }),
                Err(e) => Err(e),
            };
        }

        model.delete(self.db.as_ref()).await
    }

    pub async fn update<E>(
        &self,
        ctx: &DbContext,
        // Use whatever primary key value type Entity E requires
        id: <<E::Entity as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType,
        fill_values: impl FnOnce(&mut E),
    ) -> Result<<E::Entity as EntityTrait>::Model, DbErr>
    where
        <E::Entity as EntityTrait>::Model: IntoActiveModel<E>,
        E: Updatable + ActiveModelBehavior + Send,
    {
        let model = E::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(DbErr::RecordNotFound("Record not found".to_owned()))?;
        let mut model = model.into_active_model();

        fill_values(&mut model);
        model.fill_update_audit(ctx.id.clone());

        model.update(self.db.as_ref()).await
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

        let paginator = selector.paginate(self.db.as_ref(), items_per_page);
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
