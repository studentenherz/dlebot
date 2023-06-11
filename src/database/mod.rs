mod schema;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbBackend,
    EntityTrait, QueryFilter, Set, Statement,
};
use std::env;

use self::schema::{dle::Model as DleModel, user, word_of_the_day};
use chrono::offset::Local;
use schema::prelude::{Dle, User, WordOfTheDay};

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

    /// Get word of the day: select a random word that hasn't been WOTD and returns it
    pub async fn get_word_of_the_day(&self) -> String {
        let today = Local::now().date_naive();

        // Get a word that has today's date
        let lemma = match WordOfTheDay::find()
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
            Some(word_of_the_day::Model { lemma, .. }) => lemma,
            None => {
                // Get a random word that hasn't been WOTD
                let wotd = WordOfTheDay::find()
                    .from_raw_sql(Statement::from_string(
                        DbBackend::Postgres,
                        r#"SELECT * FROM "word_of_the_day" WHERE "date" IS NULL ORDER BY RANDOM() LIMIT 1"#
                            .to_string(),
                    ))
                    .one(&self.db)
                    .await
                    .unwrap_or_else(|x| {
                        log::error!("Error accessing the database: {:?}", x);
                        None
                    })
                    .unwrap();

                // Set it to used today
                let mut active_wotd: word_of_the_day::ActiveModel = wotd.clone().into();
                active_wotd.date = Set(Some(today));
                active_wotd.update(&self.db).await.unwrap();

                wotd.lemma
            }
        };

        // Return the definition
        self.get_exact(&lemma).await.unwrap().definition
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
            .unwrap()
    }

    /// Set subscribed status
    pub async fn set_subscribed(&self, user_id: i64, subscribed: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.subscribed = Set(subscribed);
            user.in_bot = Set(true);
            user.update(&self.db).await.unwrap();
        } else {
            let new_user = user::Model {
                id: user_id,
                subscribed,
                blocked: false,
                in_bot: true,
                admin: false,
            };
            let new_user: user::ActiveModel = new_user.into();
            new_user.insert(&self.db).await.unwrap();
        }
    }

    /// Get list of subscribed users
    pub async fn get_subscribed_and_in_bot_list(&self) -> Vec<i64> {
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

    /// Set blocked status
    /// TODO: When the admin role is added, admins should be able to
    /// ban users
    pub async fn _set_blocked(&self, user_id: i64, blocked: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.blocked = Set(blocked);
            user.update(&self.db).await.unwrap();
        }
    }

    /// Set in_bot status
    pub async fn _set_in_bot(&self, user_id: i64, in_bot: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.in_bot = Set(in_bot);
            user.update(&self.db).await.unwrap();
        }
    }

    /// Set admin status
    /// TODO:
    pub async fn _set_admin(&self, user_id: i64, admin: bool) {
        if let Some(user) = self.get_user(user_id).await {
            let mut user: user::ActiveModel = user.into();
            user.admin = Set(admin);
            user.update(&self.db).await.unwrap();
        }
    }
}
