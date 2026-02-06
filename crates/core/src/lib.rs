pub mod domain {
    pub mod entities {
        pub mod user;
    }
    pub mod ports {
        pub mod messaging;
        pub mod user_repo;
    }
}
pub mod services {
    pub mod user_service;
}
