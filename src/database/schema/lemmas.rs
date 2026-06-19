use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lemmas")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub query: String,
    pub resolved_headword: Option<String>,
    pub url: Option<String>,
    pub http_status: Option<i32>,
    pub found: bool,
    #[sea_orm(column_type = "Text", nullable)]
    pub raw_html: Option<String>,
    pub fetched_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::entries::Entity")]
    Entries,
}

impl Related<super::entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Entries.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
