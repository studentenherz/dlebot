use sea_orm_migration::{prelude::*, sea_orm::ConnectionTrait};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"
                CREATE TABLE lemmas (
                    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    query             TEXT NOT NULL UNIQUE,
                    resolved_headword TEXT,
                    url               TEXT,
                    http_status       INTEGER,
                    found             BOOLEAN NOT NULL DEFAULT FALSE,
                    raw_html          TEXT,
                    fetched_at        TIMESTAMPTZ
                );

                CREATE TABLE entries (
                    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    lemma_id        BIGINT NOT NULL REFERENCES lemmas(id) ON DELETE CASCADE,
                    headword        TEXT NOT NULL,
                    homograph       SMALLINT,
                    etymology_text  TEXT,
                    etymology_html  TEXT
                );

                CREATE TABLE sense_groups (
                    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    entry_id    BIGINT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
                    kind        TEXT NOT NULL CHECK (kind IN ('main', 'complex_form')),
                    form_text   TEXT,
                    form_level  TEXT,
                    position    SMALLINT NOT NULL
                );

                CREATE TABLE senses (
                    id               BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    sense_group_id   BIGINT NOT NULL REFERENCES sense_groups(id) ON DELETE CASCADE,
                    number           SMALLINT,
                    definition_text  TEXT,
                    definition_html  TEXT
                );

                CREATE TABLE labels (
                    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    abbr      TEXT NOT NULL,
                    full_text TEXT,
                    UNIQUE (abbr, full_text)
                );

                CREATE TABLE sense_labels (
                    sense_id  BIGINT NOT NULL REFERENCES senses(id) ON DELETE CASCADE,
                    label_id  BIGINT NOT NULL REFERENCES labels(id),
                    ordinal   SMALLINT,
                    PRIMARY KEY (sense_id, label_id)
                );

                CREATE TABLE examples (
                    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    sense_id  BIGINT NOT NULL REFERENCES senses(id) ON DELETE CASCADE,
                    text      TEXT NOT NULL,
                    position  SMALLINT NOT NULL
                );

                CREATE TABLE relations (
                    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    sense_id  BIGINT NOT NULL REFERENCES senses(id) ON DELETE CASCADE,
                    kind      TEXT NOT NULL CHECK (kind IN ('synonym', 'antonym')),
                    word      TEXT NOT NULL,
                    scope     TEXT,
                    position  SMALLINT NOT NULL
                );

                CREATE TABLE conjugations (
                    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                    entry_id  BIGINT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
                    mood      TEXT NOT NULL,
                    tense     TEXT NOT NULL,
                    person    SMALLINT,
                    number    TEXT,
                    pronoun   TEXT,
                    form      TEXT NOT NULL,
                    UNIQUE (entry_id, mood, tense, pronoun)
                );

                CREATE INDEX idx_entries_lemma      ON entries(lemma_id);
                CREATE INDEX idx_groups_entry       ON sense_groups(entry_id);
                CREATE INDEX idx_senses_group       ON senses(sense_group_id);
                CREATE INDEX idx_sense_labels_label ON sense_labels(label_id);
                CREATE INDEX idx_examples_sense     ON examples(sense_id);
                CREATE INDEX idx_relations_sense    ON relations(sense_id);
                CREATE INDEX idx_conjugations_entry ON conjugations(entry_id);
            "#)
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"
                DROP TABLE IF EXISTS conjugations  CASCADE;
                DROP TABLE IF EXISTS relations     CASCADE;
                DROP TABLE IF EXISTS examples      CASCADE;
                DROP TABLE IF EXISTS sense_labels  CASCADE;
                DROP TABLE IF EXISTS labels        CASCADE;
                DROP TABLE IF EXISTS senses        CASCADE;
                DROP TABLE IF EXISTS sense_groups  CASCADE;
                DROP TABLE IF EXISTS entries       CASCADE;
                DROP TABLE IF EXISTS lemmas        CASCADE;
            "#)
            .await
            .map(|_| ())
    }
}
