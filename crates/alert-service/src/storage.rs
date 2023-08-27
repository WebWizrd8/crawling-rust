use dyn_clone::DynClone;
use std::time::UNIX_EPOCH;
use util::convert::TryConvert;
use util::{service_registry::AlertFilter, Result};

use cosmrs::proto::traits::Message;
use mempools_api::api::{user_alert::Status, BackendUserAlert, CreateAlertRequest};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};

#[derive(Clone, Default)]
pub struct ChainFilter {
    pub id: Option<i32>,
}

#[tonic::async_trait]
pub trait AlertStorage: DynClone + Send + Sync + 'static {
    async fn create_alert(
        &self,
        req: &CreateAlertRequest,
        client_id: String,
    ) -> Result<BackendUserAlert>;
    async fn get_alerts(
        &self,
        filter: AlertFilter,
        page: Option<u64>,
    ) -> Result<Vec<BackendUserAlert>>;
    async fn update_alert(&self, alert: BackendUserAlert) -> Result<BackendUserAlert>;
    async fn delete_alert(&self, id: i32) -> Result<()>;
}
dyn_clone::clone_trait_object!(AlertStorage);

#[tonic::async_trait]
impl AlertStorage for DatabaseConnection {
    async fn create_alert(
        &self,
        req: &CreateAlertRequest,
        client_id: String,
    ) -> Result<BackendUserAlert> {
        let req = req.clone();
        let alert = req.alert.as_ref().ok_or("could not find filter in alert")?;

        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos();

        let model = db_entities::user_alert::ActiveModel {
            user_id: Set(req.user_id),
            alert: Set(hex::encode(alert.encode_to_vec())),
            message: Set(req.message),
            chain_id: Set(req.chain_id.parse::<i32>()?),
            status: Set(Status::Enabled as i32),
            alert_source: Set(alert.try_convert()?),
            name: Set(req.name.clone()),
            created_at: Set(now.to_string()),
            updated_at: Set(now.to_string()),
            client_id: Set(client_id.clone()),
            ..Default::default()
        };

        let alert = model.insert(self).await?;

        Ok(alert.try_convert()?)
    }

    async fn get_alerts(
        &self,
        filter: AlertFilter,
        page: Option<u64>,
    ) -> Result<Vec<BackendUserAlert>> {
        let mut query = db_entities::user_alert::Entity::find()
            .filter(db_entities::user_alert::Column::DeletedAt.is_null());

        if let Some(id) = filter.id {
            query = query.filter(db_entities::user_alert::Column::Id.eq(id));
        }

        if let Some(user_id) = filter.user_id {
            query = query.filter(db_entities::user_alert::Column::UserId.eq(user_id));
        }

        if let Some(chain_id) = filter.chain_id {
            query = query.filter(db_entities::user_alert::Column::ChainId.eq(chain_id));
        }

        if let Some(alert_source) = filter.alert_source {
            query =
                query.filter(db_entities::user_alert::Column::AlertSource.eq(alert_source as i32));
        }

        let models;
        if let Some(page) = page {
            models = query.paginate(self, 20).fetch_page(page).await?;
        } else {
            models = query.all(self).await?;
        }

        Ok(models.try_convert()?)
    }

    async fn update_alert(&self, req: BackendUserAlert) -> Result<BackendUserAlert> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos();
        let req = req.user_alert.ok_or("Missing user alert")?;
        let alert = db_entities::user_alert::ActiveModel {
            id: Set(req.id.parse::<i32>()?),
            message: Set(req.message),
            status: Set(req.status),
            name: Set(req.name.clone()),
            updated_at: Set(now.to_string()),
            ..Default::default()
        };

        let alert = alert.update(self).await?;

        Ok(alert.try_convert()?)
    }

    async fn delete_alert(&self, id: i32) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos();

        let alert = db_entities::user_alert::ActiveModel {
            id: Set(id),
            deleted_at: Set(Some(now.to_string())),
            ..Default::default()
        };

        alert.update(self).await?;

        Ok(())
    }
}
