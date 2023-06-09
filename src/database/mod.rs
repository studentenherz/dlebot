mod schema;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbBackend, EntityTrait, Statement};
use std::env;

use self::schema::dle::Model as DleModel;
use schema::prelude::Dle;

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
}
