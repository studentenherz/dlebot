use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sense_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub entry_id: i64,
    pub kind: String,
    pub form_text: Option<String>,
    pub form_level: Option<String>,
    pub position: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entries::Entity",
        from = "Column::EntryId",
        to = "super::entries::Column::Id"
    )]
    Entry,
    #[sea_orm(has_many = "super::senses::Entity")]
    Senses,
}

impl Related<super::entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Entry.def()
    }
}

impl Related<super::senses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Senses.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
