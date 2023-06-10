mod schema;

use sea_orm::{
    ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, DbBackend, EntityTrait, Set,
    Statement,
};
use std::env;

use self::schema::{dle::Model as DleModel, word_of_the_day};
use chrono::offset::Local;
use schema::prelude::{Dle, WordOfTheDay};

#[derive(Clone)]
pub struct DatabaseHandler {
    db: DatabaseConnection,
}

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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
        {
            Some(word_of_the_day::Model { lemma, .. }) => lemma,
            None => {
                // Get a word that hasn't been WOTD
                let wotd = WordOfTheDay::find()
                    .from_raw_sql(Statement::from_string(
                        DbBackend::Postgres,
                        r#"SELECT * FROM "word_of_the_day" WHERE "date" IS NULL LIMIT 1"#
                            .to_string(),
                    ))
                    .one(&self.db)
                    .await
                    .unwrap()
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
