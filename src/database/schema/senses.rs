use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "senses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub sense_group_id: i64,
    pub number: Option<i16>,
    #[sea_orm(column_type = "Text", nullable)]
    pub definition_text: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub definition_html: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sense_groups::Entity",
        from = "Column::SenseGroupId",
        to = "super::sense_groups::Column::Id"
    )]
    SenseGroup,
    #[sea_orm(has_many = "super::examples::Entity")]
    Examples,
    #[sea_orm(has_many = "super::word_relations::Entity")]
    WordRelations,
}

impl Related<super::sense_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SenseGroup.def()
    }
}

impl Related<super::examples::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Examples.def()
    }
}

impl Related<super::word_relations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WordRelations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
