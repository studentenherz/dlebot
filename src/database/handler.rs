use std::collections::{HashMap, HashSet};
use std::env;

use chrono::{offset::Local, NaiveDate};
use sea_orm::{
    entity::prelude::DateTimeWithTimeZone, sea_query::Expr, ActiveModelTrait, ColumnTrait,
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, Statement, Value,
};

use crate::dle_ir::{DleEntry, DleExample, DleRelation, DleSense, DleSenseGroup, DleWord};
use crate::settings::{MessageFormat, UserSettings};

use super::schema::{event, lemmas, sea_orm_active_enums::EventType, user, word_of_the_day};

#[derive(Clone)]
pub struct DatabaseHandler {
    db: DatabaseConnection,
}

impl DatabaseHandler {
    pub async fn new(uri: String) -> Self {
        let mut opt = ConnectOptions::new(uri);
        opt.sqlx_logging(false);
        let db = Database::connect(opt).await.unwrap();
        DatabaseHandler { db }
    }

    pub async fn from_env() -> Self {
        Self::new(env::var("DATABASE_URL").unwrap()).await
    }
}

fn make_placeholders(n: usize) -> String {
    (1..=n)
        .map(|i| format!("${i}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl DatabaseHandler {
    async fn load_words(&self, lemma_rows: Vec<lemmas::Model>) -> Vec<DleWord> {
        if lemma_rows.is_empty() {
            return vec![];
        }

        let lemma_ids: Vec<i64> = lemma_rows.iter().map(|l| l.id).collect();

        let structural_rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &format!(
                    "SELECT \
                        e.lemma_id, \
                        e.id AS entry_id,     e.headword,          e.homograph, \
                        e.etymology_text,     e.etymology_html, \
                        sg.id AS group_id,    sg.kind AS group_kind, sg.form_text, \
                        s.id AS sense_id,     s.number AS sense_number, \
                        s.definition_text,    s.definition_html \
                     FROM entries e \
                     JOIN sense_groups sg ON sg.entry_id      = e.id \
                     LEFT JOIN senses  s  ON s.sense_group_id = sg.id \
                     WHERE e.lemma_id IN ({}) \
                     ORDER BY e.lemma_id, e.id, sg.position, s.number",
                    make_placeholders(lemma_ids.len())
                ),
                lemma_ids
                    .iter()
                    .map(|&id| id.into())
                    .collect::<Vec<Value>>(),
            ))
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error fetching structural data: {:?}", e);
                vec![]
            });

        let mut sense_ids: Vec<i64> = Vec::new();
        {
            let mut seen = HashSet::new();
            for row in &structural_rows {
                let sid: i64 = row.try_get("", "sense_id").unwrap_or(0);
                if sid != 0 && seen.insert(sid) {
                    sense_ids.push(sid);
                }
            }
        }

        let mut examples_by_sense: HashMap<i64, Vec<DleExample>> = HashMap::new();
        let mut relations_by_sense: HashMap<i64, Vec<DleRelation>> = HashMap::new();

        if !sense_ids.is_empty() {
            let sp = make_placeholders(sense_ids.len());
            let sv: Vec<Value> = sense_ids.iter().map(|&id| id.into()).collect();

            let (examples_res, relations_res) = tokio::join!(
                self.db.query_all(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &format!(
                        "SELECT sense_id, text FROM examples \
                         WHERE sense_id IN ({sp}) \
                         ORDER BY sense_id, position"
                    ),
                    sv.clone(),
                )),
                self.db.query_all(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &format!(
                        "SELECT sense_id, kind, word, homograph, scope FROM relations \
                         WHERE sense_id IN ({sp}) \
                         ORDER BY sense_id, position"
                    ),
                    sv,
                )),
            );

            match examples_res {
                Ok(rows) => {
                    for row in rows {
                        let sense_id: i64 = row.try_get("", "sense_id").unwrap_or(0);
                        examples_by_sense
                            .entry(sense_id)
                            .or_default()
                            .push(DleExample {
                                text: row.try_get("", "text").unwrap_or_default(),
                            });
                    }
                }
                Err(e) => log::error!("DB error fetching examples: {:?}", e),
            }
            match relations_res {
                Ok(rows) => {
                    for row in rows {
                        let sense_id: i64 = row.try_get("", "sense_id").unwrap_or(0);
                        relations_by_sense
                            .entry(sense_id)
                            .or_default()
                            .push(DleRelation {
                                kind: row.try_get("", "kind").unwrap_or_default(),
                                word: row.try_get("", "word").unwrap_or_default(),
                                homograph: row.try_get("", "homograph").unwrap_or(None),
                                scope: row.try_get("", "scope").unwrap_or(None),
                            });
                    }
                }
                Err(e) => log::error!("DB error fetching relations: {:?}", e),
            }
        }

        let mut entry_ids_by_lemma: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut entry_fields: HashMap<i64, (String, Option<i16>, Option<String>, Option<String>)> =
            HashMap::new();

        let mut group_ids_by_entry: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut group_fields: HashMap<i64, (String, Option<String>)> = HashMap::new();

        let mut sense_ids_by_group: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut sense_fields: HashMap<i64, (Option<i16>, Option<String>, Option<String>)> =
            HashMap::new();

        {
            let mut seen_entries: HashSet<i64> = HashSet::new();
            let mut seen_groups: HashSet<i64> = HashSet::new();

            for row in structural_rows {
                let lemma_id: i64 = row.try_get("", "lemma_id").unwrap_or(0);
                let entry_id: i64 = row.try_get("", "entry_id").unwrap_or(0);
                let group_id: i64 = row.try_get("", "group_id").unwrap_or(0);
                let sense_id: i64 = row.try_get("", "sense_id").unwrap_or(0);

                if seen_entries.insert(entry_id) {
                    entry_ids_by_lemma
                        .entry(lemma_id)
                        .or_default()
                        .push(entry_id);
                    entry_fields.insert(
                        entry_id,
                        (
                            row.try_get("", "headword").unwrap_or_default(),
                            row.try_get("", "homograph").unwrap_or(None),
                            row.try_get("", "etymology_text").unwrap_or(None),
                            row.try_get("", "etymology_html").unwrap_or(None),
                        ),
                    );
                }

                if seen_groups.insert(group_id) {
                    group_ids_by_entry
                        .entry(entry_id)
                        .or_default()
                        .push(group_id);
                    group_fields.insert(
                        group_id,
                        (
                            row.try_get("", "group_kind").unwrap_or_default(),
                            row.try_get("", "form_text").unwrap_or(None),
                        ),
                    );
                }

                if sense_id != 0 {
                    sense_ids_by_group
                        .entry(group_id)
                        .or_default()
                        .push(sense_id);
                    sense_fields.insert(
                        sense_id,
                        (
                            row.try_get("", "sense_number").unwrap_or(None),
                            row.try_get("", "definition_text").unwrap_or(None),
                            row.try_get("", "definition_html").unwrap_or(None),
                        ),
                    );
                }
            }
        }

        lemma_rows
            .into_iter()
            .map(|lemma| {
                let entries = entry_ids_by_lemma
                    .remove(&lemma.id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|entry_id| {
                        let (headword, homograph, etymology_text, etymology_html) =
                            entry_fields.remove(&entry_id).unwrap_or_default();
                        let sense_groups = group_ids_by_entry
                            .remove(&entry_id)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|group_id| {
                                let (kind, form_text) =
                                    group_fields.remove(&group_id).unwrap_or_default();
                                let senses = sense_ids_by_group
                                    .remove(&group_id)
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|sense_id| {
                                        let (number, definition_text, definition_html) =
                                            sense_fields.remove(&sense_id).unwrap_or_default();
                                        DleSense {
                                            number,
                                            definition_text,
                                            definition_html,
                                            examples: examples_by_sense
                                                .remove(&sense_id)
                                                .unwrap_or_default(),
                                            relations: relations_by_sense
                                                .remove(&sense_id)
                                                .unwrap_or_default(),
                                        }
                                    })
                                    .collect();
                                match kind.as_str() {
                                    "complex_form" => {
                                        DleSenseGroup::ComplexForm { form_text, senses }
                                    }
                                    _ => DleSenseGroup::Main { senses },
                                }
                            })
                            .collect();
                        DleEntry {
                            headword,
                            homograph,
                            etymology_text,
                            etymology_html,
                            sense_groups,
                        }
                    })
                    .collect();
                DleWord {
                    query: lemma.query,
                    resolved_headword: lemma.resolved_headword,
                    entries,
                }
            })
            .collect()
    }
}

impl DatabaseHandler {
    pub async fn get_list_like(&self, query: &str) -> Vec<DleWord> {
        let lemma_rows = lemmas::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id, query, resolved_headword, url, http_status, found, raw_html, fetched_at
                   FROM lemmas
                   WHERE query ILIKE $1 AND found = TRUE
                   ORDER BY query ASC
                   LIMIT 10"#,
                [(format!("{}%", query)).into()],
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                vec![]
            });
        self.load_words(lemma_rows).await
    }

    pub async fn get_fuzzy_list(&self, word: &str) -> Vec<String> {
        lemmas::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id, query, resolved_headword, url, http_status, found, raw_html, fetched_at
                   FROM lemmas
                   WHERE levenshtein(LOWER(query), LOWER($1)) < 2 AND found = TRUE
                   ORDER BY query
                   LIMIT 5"#,
                [word.into()],
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                vec![]
            })
            .into_iter()
            .map(|row| row.query)
            .collect()
    }

    pub async fn get_exact(&self, query: &str) -> Option<DleWord> {
        let lemma_rows = lemmas::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id, query, resolved_headword, url, http_status, found, raw_html, fetched_at
                   FROM lemmas
                   WHERE query ILIKE $1 AND found = TRUE
                   LIMIT 1"#,
                [query.into()],
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                vec![]
            });
        self.load_words(lemma_rows).await.into_iter().next()
    }

    pub async fn get_random(&self) -> Option<DleWord> {
        let lemma_rows = lemmas::Entity::find()
            .from_raw_sql(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT id, query, resolved_headword, url, http_status, found, raw_html, fetched_at
                   FROM lemmas
                   WHERE found = TRUE
                   ORDER BY RANDOM()
                   LIMIT 1"#
                    .to_string(),
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                vec![]
            });
        self.load_words(lemma_rows).await.into_iter().next()
    }

    pub async fn set_word_of_the_day(&self, lemma: &str, date: NaiveDate) -> Result<bool, DbErr> {
        word_of_the_day::Entity::update_many()
            .col_expr(
                word_of_the_day::Column::Date,
                Expr::value(Value::ChronoDate(None)),
            )
            .filter(word_of_the_day::Column::Date.eq(date))
            .exec(&self.db)
            .await?;

        if let Some(wotd) = word_of_the_day::Entity::find()
            .filter(word_of_the_day::Column::Lemma.eq(lemma))
            .one(&self.db)
            .await?
        {
            let mut active: word_of_the_day::ActiveModel = wotd.into();
            active.date = Set(Some(date));
            active.update(&self.db).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_word_of_the_day_schedule(&self) -> Result<Vec<word_of_the_day::Model>, DbErr> {
        word_of_the_day::Entity::find()
            .limit(10)
            .filter(word_of_the_day::Column::Date.is_not_null())
            .filter(word_of_the_day::Column::Date.gte(Local::now().date_naive()))
            .order_by_asc(word_of_the_day::Column::Date)
            .all(&self.db)
            .await
    }

    pub async fn get_word_of_the_day(&self) -> Result<DleWord, &'static str> {
        let today = Local::now().date_naive();

        let wotd = word_of_the_day::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT * FROM "word_of_the_day" WHERE "date" = $1 LIMIT 1"#,
                [today.into()],
            ))
            .one(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                None
            });

        match wotd {
            Some(word_of_the_day::Model { lemma, .. }) => self
                .get_exact(&lemma)
                .await
                .ok_or("Error obtaining word of the day"),
            None => Err("No word of the day for today"),
        }
    }
}

impl DatabaseHandler {
    pub async fn get_user(&self, user_id: i64) -> Option<user::Model> {
        super::schema::prelude::User::find()
            .filter(user::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                None
            })
    }

    pub async fn is_admin(&self, user_id: i64) -> bool {
        self.get_user(user_id)
            .await
            .map(|u| u.admin)
            .unwrap_or_default()
    }

    pub async fn _get_subscribed_and_in_bot_list(&self) -> Vec<i64> {
        super::schema::prelude::User::find()
            .filter(
                user::Column::Subscribed
                    .eq(true)
                    .and(user::Column::InBot.eq(true)),
            )
            .all(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                vec![]
            })
            .iter()
            .map(|m| m.id)
            .collect()
    }

    pub async fn get_in_bot_list(&self) -> Vec<i64> {
        super::schema::prelude::User::find()
            .filter(user::Column::InBot.eq(true))
            .all(&self.db)
            .await
            .unwrap_or_else(|e| {
                log::error!("DB error: {:?}", e);
                vec![]
            })
            .iter()
            .map(|m| m.id)
            .collect()
    }

    pub async fn set_subscribed(&self, user_id: i64, subscribed: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut u: user::ActiveModel = user.into();
            u.subscribed = Set(subscribed);
            u.in_bot = Set(true);
            if let Err(e) = u.update(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        } else {
            let new_user = user::Model {
                id: user_id,
                subscribed,
                blocked: false,
                in_bot: true,
                admin: false,
                rich_text: MessageFormat::default().is_rich(),
            };
            let new_user: user::ActiveModel = new_user.into();
            if let Err(e) = new_user.insert(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        }
    }

    /// Settings of `user_id`, or the defaults if they have no row yet.
    pub async fn get_settings(&self, user_id: i64) -> UserSettings {
        self.get_user(user_id)
            .await
            .map(|u| UserSettings {
                format: MessageFormat::from_rich_text(u.rich_text),
            })
            .unwrap_or_default()
    }

    pub async fn set_message_format(&self, user_id: i64, format: MessageFormat) {
        let rich_text = format.is_rich();

        if let Some(user) = self.get_user(user_id).await {
            let mut u: user::ActiveModel = user.into();
            u.rich_text = Set(rich_text);
            if let Err(e) = u.update(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        } else {
            let new_user = user::Model {
                id: user_id,
                subscribed: false,
                blocked: false,
                in_bot: true,
                admin: false,
                rich_text,
            };
            let new_user: user::ActiveModel = new_user.into();
            if let Err(e) = new_user.insert(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        }
    }

    pub async fn _set_blocked(&self, user_id: i64, blocked: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut u: user::ActiveModel = user.into();
            u.blocked = Set(blocked);
            if let Err(e) = u.update(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        }
    }

    pub async fn set_in_bot(&self, user_id: i64, in_bot: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut u: user::ActiveModel = user.into();
            u.in_bot = Set(in_bot);
            if let Err(e) = u.update(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        }
    }

    pub async fn _set_admin(&self, user_id: i64, admin: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut u: user::ActiveModel = user.into();
            u.admin = Set(admin);
            if let Err(e) = u.update(&self.db).await {
                log::error!("DB error: {:?}", e);
            }
        }
    }
}

impl DatabaseHandler {
    pub async fn add_message_event(
        &self,
        user_id: i64,
        date: DateTimeWithTimeZone,
        message_text: String,
    ) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::Message),
            date: Set(Some(date)),
            message_text: Set(Some(message_text)),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }

    pub async fn add_edited_message_event(
        &self,
        user_id: i64,
        date: DateTimeWithTimeZone,
        message_text: String,
    ) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::EditedMessage),
            date: Set(Some(date)),
            message_text: Set(Some(message_text)),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }

    pub async fn add_callback_query_event(&self, user_id: i64, callback_data: String) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::CallbackQuery),
            callback_data: Set(Some(callback_data)),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }

    pub async fn add_sent_definition_event(
        &self,
        user_id: i64,
        date: DateTimeWithTimeZone,
        lemma_sent: String,
    ) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            date: Set(Some(date)),
            event_type: Set(EventType::SentDefinition),
            lemma_sent: Set(Some(lemma_sent)),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }

    pub async fn add_chosen_inline_result_event(
        &self,
        user_id: i64,
        result_id: String,
        query: String,
    ) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::ChosenInlineResult),
            result_id: Set(Some(result_id)),
            query: Set(Some(query)),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }

    pub async fn add_user_joined_event(&self, user_id: i64) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::UserJoined),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }

    pub async fn add_user_left_event(&self, user_id: i64) {
        let ev = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::UserLeft),
            ..Default::default()
        };
        if let Err(e) = ev.insert(&self.db).await {
            log::error!("DB error: {:?}", e);
        }
    }
}
