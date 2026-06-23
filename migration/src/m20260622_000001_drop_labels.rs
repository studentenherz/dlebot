use sea_orm_migration::{prelude::*, sea_orm::ConnectionTrait};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS sense_labels CASCADE;\
                 DROP TABLE IF EXISTS labels CASCADE;",
            )
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE labels (\
                    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,\
                    abbr      TEXT NOT NULL,\
                    full_text TEXT,\
                    UNIQUE (abbr, full_text)\
                );\
                CREATE TABLE sense_labels (\
                    sense_id  BIGINT NOT NULL REFERENCES senses(id) ON DELETE CASCADE,\
                    label_id  BIGINT NOT NULL REFERENCES labels(id),\
                    ordinal   SMALLINT,\
                    PRIMARY KEY (sense_id, label_id)\
                );\
                CREATE INDEX idx_sense_labels_label ON sense_labels(label_id);",
            )
            .await
            .map(|_| ())
    }
}
