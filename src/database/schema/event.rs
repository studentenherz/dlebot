//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use super::sea_orm_active_enums::MyChatMemberUpdate;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i64,
    pub date: DateTime,
    pub chat_id: Option<i64>,
    pub message_id: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub message_text: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub result_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub query: Option<String>,
    pub edited_message: Option<bool>,
    pub my_chat_member: Option<MyChatMemberUpdate>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
