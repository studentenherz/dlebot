use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Word of the day
        manager
            .create_table(
                Table::create()
                    .table(WordOfTheDay::Table)
                    .col(
                        ColumnDef::new(WordOfTheDay::Lemma)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WordOfTheDay::Date).date().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WordOfTheDay::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum WordOfTheDay {
    Table,
    Lemma,
    Date,
}
