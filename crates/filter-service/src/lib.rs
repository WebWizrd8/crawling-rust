use log::warn;
use mempools_api::api::{user_alert::Status, BackendUserAlert};

use util::{
    service_registry::{
        AlertFilter, AlertNotification, FilterServiceInterface, Notification,
        ProcessAlertSourceRequeust, ServiceRegistry,
    },
    Result,
};

use alerts::AlertSourceFilter;

#[derive(Clone)]
pub struct FilterService {
    registry: ServiceRegistry,
}

impl FilterService {
    pub fn new(registry: ServiceRegistry) -> Result<Self> {
        Ok(Self { registry })
    }
}

#[tonic::async_trait]
impl FilterServiceInterface for FilterService {
    async fn process_alert_source(&self, alert_source: ProcessAlertSourceRequeust) -> Result<()> {
        let registry = self.registry.get_services().await?;
        let alert_service = registry.alert_service;

        let mut page = 0;
        loop {
            let ctx = alert_source.ctx();
            let filter = AlertFilter {
                alert_source: Some(ctx.source_type),
                chain_id: Some(ctx.chain_id.parse::<i32>()?),
                ..Default::default()
            };
            let alerts = alert_service.get_alerts(filter, Some(page)).await?;

            if alerts.is_empty() {
                break;
            }

            for alert in alerts {
                let user_alert_id = alert
                    .user_alert
                    .as_ref()
                    .ok_or("Missing user alert")?
                    .id
                    .clone();

                let ctx = ctx.clone();
                let svc = registry.filter_service.clone();
                let alert_source = alert_source.clone();
                let alert_source_id = ctx.id;
                let alert_source_type_name = ctx.source_type.as_str_name();
                tokio::spawn(async move {
                    if let Err(err) = svc
                        .process_alert_for_alert_source(alert_source, alert.clone())
                        .await
                    {
                        warn!(
                            "failed to filter alert source {} {}, for alert id {},reason - {}",
                            alert_source_type_name, alert_source_id, user_alert_id, err
                        )
                    };
                });
            }

            page += 1;
        }

        Ok(())
    }

    async fn process_alert_for_alert_source(
        &self,
        alert_source: ProcessAlertSourceRequeust,
        alert: BackendUserAlert,
    ) -> Result<()> {
        let user_alert = alert.user_alert.ok_or("Missing user alert")?;

        let ctx = alert_source.ctx();
        let registry = self.registry.get_services().await?;

        if user_alert.status == Status::Disabled as i32 {
            return Ok(());
        }

        let tx_alert: Box<dyn AlertSourceFilter> = user_alert.clone().try_into()?;
        let notification = if let Ok(notification) = tx_alert.filter(&alert_source) {
            notification
        } else {
            return Ok(());
        };

        registry
            .notification_service
            .send_notification(
                Notification::AlertNotification(AlertNotification {
                    notification,
                    alert_id: user_alert.id.clone(),
                    alert_source_id: ctx.id,
                }),
                user_alert.user_id,
            )
            .await?;

        Ok(())
    }
}
