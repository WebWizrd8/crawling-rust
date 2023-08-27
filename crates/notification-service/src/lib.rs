use std::time::Instant;

use reqwest::Client;
use util::service_registry::{NotificationStatistics, ResponseTime, ServiceRegistry};
use util::{
    service_registry::{
        AlertNotification, Notification, NotificationFilter, NotificationServiceInterface,
    },
    Result,
};

use self::storage::NotificationStorage;

pub mod storage;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArchxConnector {
    user_id: String,
    alert: AlertNotification,
}

#[derive(Clone)]
pub struct NotificationService {
    registry: ServiceRegistry,
    storage: Box<dyn NotificationStorage>,
    http_client: reqwest::Client,
}

#[tonic::async_trait]
impl NotificationServiceInterface for NotificationService {
    async fn send_notification(
        &self,
        notification: Notification,
        alert_owner_id: String,
    ) -> Result<()> {
        match notification {
            Notification::AlertNotification(n) => {
                self.send_alert_notification(n, alert_owner_id).await?
            }
        }

        Ok(())
    }
    async fn get_statistics(
        &self,
        alert_id: Option<i32>,
        user_id: Option<String>,
    ) -> Result<NotificationStatistics> {
        self.storage.get_statistics(alert_id, user_id).await
    }
    async fn get_notifications(
        &self,
        filter: NotificationFilter,
        page: u64,
    ) -> Result<Vec<mempools_api::api::AlertNotification>> {
        self.storage.get_notifications(filter, Some(page)).await
    }
}

impl NotificationService {
    pub fn new<S: NotificationStorage>(registry: ServiceRegistry, storage: S) -> Result<Self> {
        Ok(Self {
            registry,
            storage: Box::new(storage),
            http_client: Client::new(),
        })
    }
    async fn send_alert_notification(
        &self,
        alert_notification: AlertNotification,
        user_id: String,
    ) -> Result<()> {
        let request_start = Instant::now();

        let client_id = self
            .registry
            .get_services()
            .await?
            .alert_service
            .get_alert_by_id(alert_notification.alert_id.clone())
            .await?
            .client_id;
        self.http_client
            .post(&self.storage.get_endpoint(&client_id).await?)
            .json(&ArchxConnector {
                user_id,
                alert: alert_notification.clone(),
            })
            .send()
            .await?;
        let response_times: ResponseTime = ResponseTime {
            total_response_time: request_start.elapsed(),
            num_responses: 1,
        };
        self.storage
            .create_notification(&alert_notification, response_times)
            .await?;
        Ok(())
    }
}
