use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "conjugations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub entry_id: i64,
    pub mood: String,
    pub tense: String,
    pub person: Option<i16>,
    pub number: Option<String>,
    pub pronoun: Option<String>,
    pub form: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entries::Entity",
        from = "Column::EntryId",
        to = "super::entries::Column::Id"
    )]
    Entry,
}

impl Related<super::entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Entry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
