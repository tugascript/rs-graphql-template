pub use sea_orm_migration::prelude::*;

mod m20230922_000001_create_user_table;
mod m20230922_000002_create_token_blacklist_table;
mod m20230922_000003_create_oauth_provider_table;
mod m20230923_000004_create_access_code_table;
mod m20230923_000005_create_csrf_token_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230922_000001_create_user_table::Migration),
            Box::new(m20230922_000002_create_token_blacklist_table::Migration),
            Box::new(m20230922_000003_create_oauth_provider_table::Migration),
            Box::new(m20230923_000004_create_access_code_table::Migration),
            Box::new(m20230923_000005_create_csrf_token_table::Migration),
        ]
    }
}
