use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Dictionary of words
        manager
            .create_table(
                Table::create()
                    .table(Dle::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Dle::Lemma).string().not_null().primary_key())
                    .col(ColumnDef::new(Dle::Definition).text().not_null())
                    .col(ColumnDef::new(Dle::Conjugation).json_binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Dle::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Dle {
    Table,
    Lemma,
    Definition,
    Conjugation,
}
