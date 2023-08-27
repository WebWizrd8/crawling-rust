//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alert_notification")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub notification_data: String,
    pub alert_id: i32,
    pub alert_source_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    #[sea_orm(column_type = "Float")]
    pub total_response_time: f32,
    pub num_responses: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_alert::Entity",
        from = "Column::AlertId",
        to = "super::user_alert::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    UserAlert,
}

impl Related<super::user_alert::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAlert.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
