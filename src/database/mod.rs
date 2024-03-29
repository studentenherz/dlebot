mod schema;

use sea_orm::{
    entity::prelude::DateTimeWithTimeZone, sea_query::Expr, ActiveModelTrait, ColumnTrait,
    ConnectOptions, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, Statement, Value,
};
use std::env;

use chrono::{offset::Local, NaiveDate};
use schema::{
    event,
    prelude::{Dle, User, WordOfTheDay},
    sea_orm_active_enums::EventType,
    user, word_of_the_day,
};

pub type DleModel = schema::dle::Model;

#[derive(Clone)]
pub struct DatabaseHandler {
    db: DatabaseConnection,
}

/// Dictionary implementations
impl DatabaseHandler {
    /// Get handler from uri
    pub async fn new(uri: String) -> Self {
        let mut opt = ConnectOptions::new(uri);
        opt.sqlx_logging(false);

        let db = Database::connect(opt).await.unwrap();

        DatabaseHandler { db }
    }

    /// Get handler with uri from the DATABASE_URL environment variable
    pub async fn from_env() -> Self {
        Self::new(env::var("DATABASE_URL").unwrap()).await
    }

    /// Get list of the 10 first rows whose "lemma" starts with `query`.
    /// This is case insensitive.
    pub async fn get_list_like(&self, query: &str) -> Vec<DleModel> {
        Dle::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT * FROM "dle" WHERE "dle"."lemma" ILIKE $1 ORDER BY "dle"."lemma" ASC LIMIT 10"#,
                [(format!("{}%", query)).into()],
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                vec![]
            })
    }

    /// Get list of the 5 first lemmas that match `word` within distance of two.
    /// This is case insensitive.
    pub async fn get_fuzzy_list(&self, word: &str) -> Vec<String> {
        Dle::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT * FROM "dle" WHERE levenshtein(LOWER("lemma"), LOWER($1)) < 2 ORDER BY "lemma" LIMIT 5"#,
                [word.into()],
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                vec![]
            })
            .iter()
            .map(|row| row.lemma.clone())
            .collect()
    }

    /// Get row with "lemma" == `lemma`. Case insensitive.
    pub async fn get_exact(&self, lemma: &str) -> Option<DleModel> {
        Dle::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT * FROM "dle" WHERE "dle"."lemma" ILIKE $1 LIMIT 1"#,
                [lemma.into()],
            ))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    /// Get random word
    pub async fn get_random(&self) -> Option<DleModel> {
        Dle::find()
            .from_raw_sql(Statement::from_string(
                DbBackend::Postgres,
                r#"SELECT * FROM "dle" ORDER BY RANDOM() LIMIT 1"#.to_string(),
            ))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    /// Set word of the day
    pub async fn set_word_of_the_day(&self, lemma: &str, date: NaiveDate) -> Result<bool, DbErr> {
        WordOfTheDay::update_many()
            .col_expr(
                word_of_the_day::Column::Date,
                Expr::value(Value::ChronoDate(None)),
            )
            .filter(word_of_the_day::Column::Date.eq(date))
            .exec(&self.db)
            .await?;

        if let Some(wotd) = WordOfTheDay::find()
            .filter(word_of_the_day::Column::Lemma.eq(lemma))
            .one(&self.db)
            .await?
        {
            let mut active_wotd: word_of_the_day::ActiveModel = wotd.into();
            active_wotd.date = Set(Some(date));
            active_wotd.update(&self.db).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Get schedule
    pub async fn get_word_of_the_day_schedule(&self) -> Result<Vec<word_of_the_day::Model>, DbErr> {
        WordOfTheDay::find()
            .limit(10)
            .filter(word_of_the_day::Column::Date.is_not_null())
            .filter(word_of_the_day::Column::Date.gte(Local::now().date_naive()))
            .order_by_asc(word_of_the_day::Column::Date)
            .all(&self.db)
            .await
    }

    /// Get word of the day: select a random word that hasn't been WOTD and returns it
    pub async fn get_word_of_the_day(&self) -> Result<DleModel, &'static str> {
        let today = Local::now().date_naive();

        // Get a word that has today's date
        match WordOfTheDay::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT * FROM "word_of_the_day" WHERE "date" = $1 LIMIT 1"#,
                [today.into()],
            ))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            }) {
            Some(word_of_the_day::Model { lemma, .. }) => {
                if let Some(result) = self.get_exact(&lemma).await {
                    return Ok(result);
                }
            }
            None => return Err("No word of the day for today"),
        }

        Err("Error obtaining word of the day")
    }
}

/// User implementations
impl DatabaseHandler {
    /// Get user
    pub async fn get_user(&self, user_id: i64) -> Option<user::Model> {
        User::find()
            .filter(user::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    /// Is admin?
    pub async fn is_admin(&self, user_id: i64) -> bool {
        self.get_user(user_id)
            .await
            .map(|user| user.admin)
            .unwrap_or_default()
    }

    /// Get list of subscribed users
    pub async fn _get_subscribed_and_in_bot_list(&self) -> Vec<i64> {
        User::find()
            .filter(
                user::Column::Subscribed
                    .eq(true)
                    .and(user::Column::InBot.eq(true)),
            )
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                vec![]
            })
            .iter()
            .map(|m| m.id)
            .collect()
    }

    /// Get list of in-bot users
    pub async fn get_in_bot_list(&self) -> Vec<i64> {
        User::find()
            .filter(user::Column::InBot.eq(true))
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                vec![]
            })
            .iter()
            .map(|m| m.id)
            .collect()
    }

    /// Set subscribed status
    pub async fn set_subscribed(&self, user_id: i64, subscribed: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.subscribed = Set(subscribed);
            user.in_bot = Set(true);
            if let Err(x) = user.update(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        } else {
            let new_user = user::Model {
                id: user_id,
                subscribed,
                blocked: false,
                in_bot: true,
                admin: false,
            };
            let new_user: user::ActiveModel = new_user.into();
            if let Err(x) = new_user.insert(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        }
    }

    /// Set blocked status
    /// TODO: When the admin role is added, admins should be able to
    /// ban users
    pub async fn _set_blocked(&self, user_id: i64, blocked: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.blocked = Set(blocked);
            if let Err(x) = user.update(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        }
    }

    /// Set in_bot status
    pub async fn set_in_bot(&self, user_id: i64, in_bot: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.in_bot = Set(in_bot);
            if let Err(x) = user.update(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        }
    }

    /// Set admin status
    /// TODO:
    pub async fn _set_admin(&self, user_id: i64, admin: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.admin = Set(admin);
            if let Err(x) = user.update(&self.db).await {
                log::error!("Error accessing the database: {:?}", x);
            }
        }
    }
}

/// Event implementations
impl DatabaseHandler {
    pub async fn add_message_event(
        &self,
        user_id: i64,
        date: DateTimeWithTimeZone,
        message_text: String,
    ) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::Message),
            date: Set(Some(date)),
            message_text: Set(Some(message_text)),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }

    pub async fn add_edited_message_event(
        &self,
        user_id: i64,
        date: DateTimeWithTimeZone,
        message_text: String,
    ) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::EditedMessage),
            date: Set(Some(date)),
            message_text: Set(Some(message_text)),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }

    pub async fn add_callback_query_event(&self, user_id: i64, callback_data: String) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::CallbackQuery),
            callback_data: Set(Some(callback_data)),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }

    pub async fn add_sent_definition_event(
        &self,
        user_id: i64,
        date: DateTimeWithTimeZone,
        lemma_sent: String,
    ) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            date: Set(Some(date)),
            event_type: Set(EventType::SentDefinition),
            lemma_sent: Set(Some(lemma_sent)),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }

    pub async fn add_chosen_inline_result_event(
        &self,
        user_id: i64,
        result_id: String,
        query: String,
    ) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::ChosenInlineResult),
            result_id: Set(Some(result_id)),
            query: Set(Some(query)),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }

    pub async fn add_user_joined_event(&self, user_id: i64) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::UserJoined),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }

    pub async fn add_user_left_event(&self, user_id: i64) {
        let new_event = event::ActiveModel {
            user_id: Set(user_id),
            event_type: Set(EventType::UserLeft),
            ..Default::default()
        };

        if let Err(x) = new_event.insert(&self.db).await {
            log::error!("Error accessing the database: {:?}", x);
        };
    }
}
