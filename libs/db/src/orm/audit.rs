use sea_orm::ActiveModelTrait;

pub trait Creatable: ActiveModelTrait {
    fn fill_create_audit(&mut self, user_id: String);
}

pub trait Updatable: ActiveModelTrait {
    fn fill_update_audit(&mut self, user_id: String);
}

pub trait Deletable: ActiveModelTrait {
    fn fill_delete_audit(&mut self, user_id: String);
    fn should_be_soft(&self) -> bool;
}

#[macro_export]
macro_rules! make_creatable {
    ($model:ty) => {
        impl $crate::orm::audit::Creatable for $model {
            fn fill_create_audit(&mut self, user_id: String) {
                use sea_orm::{
                    ActiveModelTrait, ActiveValue::Set, EntityTrait, Iden, Iterable, Value,
                };

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

#[macro_export]
macro_rules! make_updatable {
    ($model:ty) => {
        impl $crate::orm::audit::Updatable for $model {
            fn fill_update_audit(&mut self, user_id: String) {
                use sea_orm::ActiveValue::Set;

                self.updated_at = Set(Some(chrono::Utc::now().into()));
                self.updated_by = Set(Some(user_id));
            }
        }
    };
}

#[macro_export]
macro_rules! make_deletable {
    ($model:ty) => {
        impl $crate::orm::audit::Deletable for $model {
            fn fill_delete_audit(&mut self, user_id: String) {
                use sea_orm::{
                    ActiveModelTrait, ActiveValue::Set, EntityTrait, Iden, Iterable, Value,
                };

                let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

                for col in <<$model as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Column::iter()
                {
                    match col.to_string().as_str() {
                        "deleted_at" => {
                            self.set(col, Value::from(Some(now)));
                        }
                        "deleted_by" => {
                            self.set(col, Value::from(Some(user_id.clone())));
                        }
                        _ => {}
                    }
                }
            }

            fn should_be_soft(&self) -> bool {
                use sea_orm::{Iden, Iterable};

                let mut is_soft = false;

                for col in <<$model as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Column::iter()
                {
                    match col.to_string().as_str() {
                        "deleted_at" => {
                            is_soft = true;
                            break;
                        }
                        "deleted_by" => {
                            is_soft = true;
                            break;
                        }
                        _ => {}
                    }
                };

                is_soft
            }
        }
    };
}
