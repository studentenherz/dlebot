mod schema;

use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbBackend, EntityTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Statement,
};
use std::env;

use self::schema::dle::{Column as DleColumn, Model as DleModel};
use schema::prelude::Dle;

#[derive(Clone)]
pub struct DatabaseHandler {
    db: DatabaseConnection,
}

impl DatabaseHandler {
    pub async fn new(uri: String) -> Self {
        let opt = ConnectOptions::new(uri);
        let db = Database::connect(opt).await.unwrap();

        DatabaseHandler { db }
    }

    pub async fn from_env() -> Self {
        Self::new(env::var("DATABASE_URL").unwrap()).await
    }

    pub async fn get_list_like(&self, query: &str) -> Vec<DleModel> {
        Dle::find()
            .filter(DleColumn::Lemma.like(&format!("{}%", query)))
            .order_by(DleColumn::Lemma, Order::Asc)
            .limit(10)
            .all(&self.db)
            .await
            .unwrap()
    }

    pub async fn get_exact(&self, lemma: &str) -> Option<DleModel> {
        Dle::find()
            .filter(DleColumn::Lemma.eq(lemma))
            .one(&self.db)
            .await
            .unwrap()
    }

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
