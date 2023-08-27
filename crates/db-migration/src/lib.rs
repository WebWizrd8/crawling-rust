use std::error::Error;

pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20230318_044041_add_chains;
mod m20230321_082824_failed_tx_alert;
mod m20230323_170133_add_chain_data;
mod m20230330_120032_add_crawler_table;
mod m20230405_044439_adding_timestamps;
mod m20230405_130847_notification_object;
mod m20230613_200442_response_time;
mod m20230725_204915_jwt_management;
mod m20230731_204843_client_webhook_endpoint;
mod m20230731_210652_client_id;
mod m20230731_220915_alert_client_id;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20230318_044041_add_chains::Migration),
            Box::new(m20230321_082824_failed_tx_alert::Migration),
            Box::new(m20230323_170133_add_chain_data::Migration),
            Box::new(m20230330_120032_add_crawler_table::Migration),
            Box::new(m20230405_044439_adding_timestamps::Migration),
            Box::new(m20230405_130847_notification_object::Migration),
            Box::new(m20230613_200442_response_time::Migration),
            Box::new(m20230725_204915_jwt_management::Migration),
            Box::new(m20230731_204843_client_webhook_endpoint::Migration),
            Box::new(m20230731_210652_client_id::Migration),
            Box::new(m20230731_220915_alert_client_id::Migration),
        ]
    }
}
pub trait ToDbResult<T> {
    fn to_db_result(self) -> core::result::Result<T, DbErr>;
}

impl<T, E: Into<Box<dyn Error + Send + Sync>>> ToDbResult<T> for core::result::Result<T, E> {
    fn to_db_result(self) -> core::result::Result<T, DbErr> {
        self.map_err(|err| {
            let err: Box<dyn Error> = err.into();
            DbErr::Custom(err.to_string())
        })
    }
}
