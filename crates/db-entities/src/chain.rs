//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "chain")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub status: i32,
    pub chain_data: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::crawler::Entity")]
    Crawler,
    #[sea_orm(has_many = "super::user_alert::Entity")]
    UserAlert,
}

impl Related<super::crawler::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Crawler.def()
    }
}

impl Related<super::user_alert::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAlert.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}