use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "examples")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub sense_id: i64,
    pub text: String,
    pub position: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::senses::Entity",
        from = "Column::SenseId",
        to = "super::senses::Column::Id"
    )]
    Sense,
}

impl Related<super::senses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sense.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
