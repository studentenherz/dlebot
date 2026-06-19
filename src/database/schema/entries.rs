use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub lemma_id: i64,
    pub headword: String,
    pub homograph: Option<i16>,
    #[sea_orm(column_type = "Text", nullable)]
    pub etymology_text: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub etymology_html: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::lemmas::Entity",
        from = "Column::LemmaId",
        to = "super::lemmas::Column::Id"
    )]
    Lemma,
    #[sea_orm(has_many = "super::sense_groups::Entity")]
    SenseGroups,
    #[sea_orm(has_many = "super::conjugations::Entity")]
    Conjugations,
}

impl Related<super::lemmas::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lemma.def()
    }
}

impl Related<super::sense_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SenseGroups.def()
    }
}

impl Related<super::conjugations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conjugations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
