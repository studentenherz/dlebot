use sea_orm_migration::prelude::*;
use sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(MyChatMemberUpdate::Table)
                    .values([MyChatMemberUpdate::Joined, MyChatMemberUpdate::Left])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Event::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Event::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Event::UserId).big_unsigned().not_null())
                    .col(ColumnDef::new(Event::Date).date_time().not_null())
                    .col(ColumnDef::new(Event::ChatId).big_integer())
                    .col(ColumnDef::new(Event::MessageId).unsigned())
                    .col(ColumnDef::new(Event::MessageText).text())
                    .col(ColumnDef::new(Event::ResultId).text())
                    .col(ColumnDef::new(Event::Query).text())
                    .col(ColumnDef::new(Event::EditedMessage).boolean())
                    .col(ColumnDef::new(Event::MyChatMember).enumeration(
                        MyChatMemberUpdate::Table,
                        [MyChatMemberUpdate::Joined, MyChatMemberUpdate::Left],
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Event::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(MyChatMemberUpdate::Table)
                    .restrict()
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Event {
    Table,
    Id,
    UserId,
    Date,
    ChatId,
    MessageId,
    MessageText,
    ResultId,
    Query,
    EditedMessage,
    MyChatMember,
}

#[derive(Iden)]
enum MyChatMemberUpdate {
    Table, // Not really a Table but better than hardcoding the Iden impl by hand
    Joined,
    Left,
}
