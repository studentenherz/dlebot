pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_dictionary;
mod m20220101_000001_create_wotd_table;
mod m20230610_033154_create_user_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_dictionary::Migration),
            Box::new(m20220101_000001_create_wotd_table::Migration),
            Box::new(m20230610_033154_create_user_table::Migration),
        ]
    }
}
