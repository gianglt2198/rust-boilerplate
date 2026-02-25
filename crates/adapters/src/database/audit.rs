use ro_db::{make_creatable, make_deletable, make_updatable};

use crate::database::entities::user;

make_creatable!(user::ActiveModel);

make_updatable!(user::ActiveModel);

make_deletable!(user::ActiveModel);
