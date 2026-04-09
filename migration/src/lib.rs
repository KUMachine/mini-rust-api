pub use sea_orm_migration::prelude::*;

pub mod m20220101_000001_create_table;
mod m20250203_000001_create_roles_table;
mod m20250203_000002_create_user_roles_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250203_000001_create_roles_table::Migration),
            Box::new(m20250203_000002_create_user_roles_table::Migration),
        ]
    }
}
