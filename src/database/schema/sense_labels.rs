use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sense_labels")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub sense_id: i64,
    #[sea_orm(primary_key)]
    pub label_id: i64,
    pub ordinal: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::senses::Entity",
        from = "Column::SenseId",
        to = "super::senses::Column::Id"
    )]
    Sense,
    #[sea_orm(
        belongs_to = "super::labels::Entity",
        from = "Column::LabelId",
        to = "super::labels::Column::Id"
    )]
    Label,
}

impl Related<super::senses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sense.def()
    }
}

impl Related<super::labels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Label.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
