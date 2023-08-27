use mempools_api::api::{BackendUserAlert, CreateAlertRequest};
use util::service_registry::{AlertFilter, AlertServiceInterface};

use util::Result;

use self::storage::AlertStorage;

pub mod storage;

#[derive(Clone)]
pub struct AlertService {
    store: Box<dyn AlertStorage>,
}

#[tonic::async_trait]
impl AlertServiceInterface for AlertService {
    async fn get_alert_by_id(&self, alert_id: String) -> Result<BackendUserAlert> {
        let alert = self
            .get_alerts(
                AlertFilter {
                    id: Some(alert_id.parse::<i32>()?),
                    ..Default::default()
                },
                None,
            )
            .await?
            .first()
            .ok_or("alert not found")?
            .clone();

        Ok(alert)
    }

    async fn get_alerts(
        &self,
        filter: AlertFilter,
        page: Option<u64>,
    ) -> Result<Vec<BackendUserAlert>> {
        self.store.get_alerts(filter, page).await
    }

    async fn create_alert(
        &self,
        req: &CreateAlertRequest,
        client_id: String,
    ) -> Result<BackendUserAlert> {
        self.store.create_alert(req, client_id).await
    }

    async fn update_alert(&self, alert: BackendUserAlert) -> Result<BackendUserAlert> {
        self.store.update_alert(alert).await
    }
    async fn delete_alert(&self, id: i32) -> Result<()> {
        self.store.delete_alert(id).await
    }
}

impl AlertService {
    pub fn new<S: AlertStorage>(store: S) -> Self {
        Self {
            store: Box::new(store),
        }
    }
}
