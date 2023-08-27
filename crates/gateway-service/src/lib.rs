use mempools_api::api::{
    gateway_server::Gateway, AlertSource, BackendUserAlert, CreateAlertRequest,
    CreateAlertResponse, DeleteAlertRequest, DeleteAlertResponse, GetAlertsRequest,
    GetAlertsResponse, GetChainsRequest, GetChainsResponse, GetNotificationsRequest,
    GetNotificationsResponse, GetStatisticsRequest, GetStatisticsResponse, SendBroadcastRequest,
    SendBroadcastResponse, UpdateAlertRequest, UpdateAlertResponse,
};

use request_validation::Validateable;
use tonic::{Request, Response, Status};
use util::{
    service_registry::{
        AlertFilter, NotificationFilter, NotificationServiceInterface, ServiceRegistry, TimeRange,
    },
    ToGrpcResult, UserMetadata,
};

pub mod admin;

#[derive(Clone)]
pub struct GatewayService {
    registry: ServiceRegistry,
}

impl GatewayService {
    pub fn new(registry: ServiceRegistry) -> Self {
        Self { registry }
    }
}

#[tonic::async_trait]
impl Gateway for GatewayService {
    async fn create_alert(
        &self,
        request: Request<CreateAlertRequest>,
    ) -> Result<Response<CreateAlertResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let alert_svc = self
            .registry
            .get_services()
            .await
            .to_grpc_result()?
            .alert_service;

        let alert = alert_svc
            .create_alert(
                request.get_ref(),
                request
                    .extensions()
                    .get::<UserMetadata>()
                    .ok_or("Missing user metadata")
                    .to_grpc_result()?
                    .client_id
                    .clone(),
            )
            .await
            .to_grpc_result()?
            .user_alert
            .ok_or("Missing user alert")
            .to_grpc_result()?;

        Ok(Response::new(CreateAlertResponse { alert: Some(alert) }))
    }

    async fn get_alerts(
        &self,
        request: Request<GetAlertsRequest>,
    ) -> Result<Response<GetAlertsResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let req = request.get_ref().clone();
        let registry = self.registry.get_services().await.to_grpc_result()?;
        let alert_svc = registry.alert_service;

        let alerts = alert_svc
            .get_alerts(
                AlertFilter {
                    user_id: req.user_id,
                    chain_id: req.chain_id,
                    ..Default::default()
                },
                req.page,
            )
            .await
            .to_grpc_result()?
            .into_iter()
            .filter_map(|a| a.user_alert)
            .collect();

        Ok(Response::new(GetAlertsResponse { alerts }))
    }
    async fn update_alert(
        &self,
        request: Request<UpdateAlertRequest>,
    ) -> Result<Response<UpdateAlertResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let req = request.get_ref().clone();
        let registry = self.registry.get_services().await.to_grpc_result()?;
        let alert_svc = registry.alert_service;

        let alert = alert_svc
            .update_alert(BackendUserAlert {
                user_alert: Some(
                    req.alert
                        .ok_or("could not find alert in request")
                        .to_grpc_result()?,
                ),
                client_id: request
                    .extensions()
                    .get::<UserMetadata>()
                    .ok_or("Missing user metadata")
                    .to_grpc_result()?
                    .client_id
                    .clone(),
            })
            .await
            .to_grpc_result()?;

        Ok(Response::new(UpdateAlertResponse {
            alert: Some(
                alert
                    .user_alert
                    .ok_or("missing user alert")
                    .to_grpc_result()?,
            ),
        }))
    }
    async fn delete_alert(
        &self,
        request: Request<DeleteAlertRequest>,
    ) -> Result<Response<DeleteAlertResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let req = request.get_ref().clone();
        let registry = self.registry.get_services().await.to_grpc_result()?;
        let alert_svc = registry.alert_service;

        alert_svc
            .delete_alert(req.alert_id.parse::<i32>().to_grpc_result()?)
            .await
            .to_grpc_result()?;

        Ok(Response::new(DeleteAlertResponse {}))
    }

    async fn get_notifications(
        &self,
        request: Request<GetNotificationsRequest>,
    ) -> Result<Response<GetNotificationsResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let req = request.get_ref().clone();
        let registry = self.registry.get_services().await.to_grpc_result()?;
        let notification_svc: Box<dyn NotificationServiceInterface> = registry.notification_service;
        let time = req.time.map(|time| TimeRange {
            start: time.start.and_then(|i| i.try_into().ok()),
            end: time.end.and_then(|i| i.try_into().ok()),
        });
        let notifications = notification_svc
            .get_notifications(
                NotificationFilter {
                    alert_id: req.alert_id.parse().ok(),
                    user_id: req.user_id,
                    time,
                    ..Default::default()
                },
                req.page,
            )
            .await
            .to_grpc_result()?;

        Ok(Response::new(GetNotificationsResponse { notifications }))
    }
    async fn get_statistics(
        &self,
        request: Request<GetStatisticsRequest>,
    ) -> Result<Response<GetStatisticsResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let req = request.get_ref().clone();
        let registry = self.registry.get_services().await.to_grpc_result()?;
        let notification_svc = registry.notification_service;
        let alert_svc = registry.alert_service;

        let stats = notification_svc
            .get_statistics(
                req.alert_id.clone().and_then(|s| s.parse().ok()),
                req.user_id,
            )
            .await
            .to_grpc_result()?;

        let creation_date = if let Some(alert_id) = &req.alert_id {
            Some(
                alert_svc
                    .get_alert_by_id(alert_id.clone())
                    .await
                    .to_grpc_result()?
                    .user_alert
                    .ok_or("Missing user alert")
                    .to_grpc_result()?
                    .created_at,
            )
        } else {
            None
        };

        let subscriber_count = if req.alert_id.is_none() {
            Some(
                alert_svc
                    .get_alerts(
                        AlertFilter {
                            id: None,
                            user_id: None,
                            chain_id: None,
                            alert_source: Some(AlertSource::ArchwaysBroadcast),
                        },
                        None,
                    )
                    .await
                    .to_grpc_result()?
                    .len()
                    .try_into()
                    .unwrap(),
            )
        } else {
            None
        };

        Ok(Response::new(GetStatisticsResponse {
            total_alerts: stats.total_alerts.into(),
            total_alerts_today: stats.total_alerts_today.into(),
            avg_response_time: stats.avg_response_time.into(),
            creation_date,
            subscriber_count,
        }))
    }
    async fn get_chains(
        &self,
        request: Request<GetChainsRequest>,
    ) -> Result<Response<GetChainsResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let registry = self.registry.get_services().await.to_grpc_result()?;
        let chain_svc = registry.chain_service;
        Ok(Response::new(
            chain_svc.get_chains().await.to_grpc_result()?,
        ))
    }
    async fn send_broadcast(
        &self,
        request: Request<SendBroadcastRequest>,
    ) -> Result<Response<SendBroadcastResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let registry = self.registry.get_services().await.to_grpc_result()?;
        let filter_svc = registry.filter_service;
        let client_id = request
            .extensions()
            .get::<UserMetadata>()
            .ok_or("Missing user metadata")
            .to_grpc_result()?
            .client_id
            .clone();
        let request = request.into_inner();
        filter_svc
            .process_alert_source(
                util::service_registry::ProcessAlertSourceRequeust::ArchwaysBroadcast {
                    chain_id: request.chain_id,
                    message: request.message,
                    client_id,
                },
            )
            .await
            .to_grpc_result()?;
        Ok(Response::new(SendBroadcastResponse {}))
    }
}
