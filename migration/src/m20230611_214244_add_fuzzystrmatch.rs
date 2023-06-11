use sea_orm_migration::{
    prelude::*,
    sea_orm::{DbBackend, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_conn = manager.get_connection();

        db_conn
            .execute(Statement::from_string(
                DbBackend::Postgres,
                r#"CREATE EXTENSION fuzzystrmatch"#.to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_conn = manager.get_connection();

        db_conn
            .execute(Statement::from_string(
                DbBackend::Postgres,
                r#"DROP EXTENSION fuzzystrmatch"#.to_string(),
            ))
            .await?;

        Ok(())
    }
}
