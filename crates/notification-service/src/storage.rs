use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cosmrs::proto::traits::Message;
use dyn_clone::DynClone;
use mempools_api::api::{AlertNotification, AlertNotificationData};
use sea_orm::sea_query::{Alias, BinOper, Expr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
    IntoActiveModel, PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use util::service_registry::{NotificationStatistics, ResponseTime};
use util::{convert::TryConvert, service_registry::NotificationFilter};

use util::Result;

#[tonic::async_trait]
pub trait NotificationStorage: DynClone + Send + Sync + 'static {
    async fn create_notification(
        &self,
        req: &super::AlertNotification,
        response_time: ResponseTime,
    ) -> Result<AlertNotification>;
    async fn get_notifications(
        &self,
        filter: NotificationFilter,
        page: Option<u64>,
    ) -> Result<Vec<AlertNotification>>;
    async fn get_statistics(
        &self,
        alert_id: Option<i32>,
        user_id: Option<String>,
    ) -> Result<NotificationStatistics>;
    async fn get_telegram_chat_id(&self, username: String) -> Result<String>;
    async fn set_telegram_chat_id(&self, username: String, chat_id: String) -> Result<()>;
    async fn get_endpoint(&self, client_id: &str) -> Result<String>;
}
dyn_clone::clone_trait_object!(NotificationStorage);

#[tonic::async_trait]
impl NotificationStorage for DatabaseConnection {
    async fn create_notification(
        &self,
        req: &super::AlertNotification,
        repsonse_time: ResponseTime,
    ) -> Result<AlertNotification> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos();

        let notification = db_entities::alert_notification::ActiveModel {
            notification_data: Set(hex::encode(
                AlertNotificationData {
                    alert_notification_data: Some(req.notification.clone()),
                }
                .encode_to_vec(),
            )),
            alert_id: Set(req.alert_id.parse::<i32>()?),
            alert_source_id: Set(req.alert_source_id.clone()),
            created_at: Set(now.to_string()),
            updated_at: Set(now.to_string()),
            total_response_time: Set(repsonse_time.total_response_time.as_secs_f32()),
            num_responses: Set(repsonse_time.num_responses.try_into()?),
            ..Default::default()
        };

        let notification = notification.insert(self).await?;

        Ok(notification.try_convert()?)
    }

    async fn get_notifications(
        &self,
        filter: NotificationFilter,
        page: Option<u64>,
    ) -> Result<Vec<AlertNotification>> {
        let mut query = db_entities::alert_notification::Entity::find()
            .filter(db_entities::alert_notification::Column::DeletedAt.is_null())
            .left_join(db_entities::user_alert::Entity)
            .filter(db_entities::user_alert::Column::UserId.eq(filter.user_id));

        if let Some(id) = filter.id {
            query = query.filter(db_entities::alert_notification::Column::Id.eq(id));
        }

        if let Some(alert_id) = filter.alert_id {
            query = query.filter(db_entities::alert_notification::Column::AlertId.eq(alert_id));
        }

        if let Some(time_range) = filter.time {
            let start = time_range.start.unwrap_or(0);
            let end = time_range.end.unwrap_or(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_nanos()
                    .try_into()?,
            );
            query = query
                .filter(
                    db_entities::alert_notification::Column::CreatedAt
                        .into_expr()
                        .cast_as(Alias::new("bigint"))
                        .binary(BinOper::GreaterThanOrEqual, Expr::value(start)),
                )
                .filter(
                    db_entities::alert_notification::Column::CreatedAt
                        .into_expr()
                        .cast_as(Alias::new("bigint"))
                        .binary(BinOper::SmallerThanOrEqual, Expr::value(end)),
                )
        }

        let models;
        if let Some(page) = page {
            models = query.paginate(self, 20).fetch_page(page).await?;
        } else {
            models = query.all(self).await?;
        }

        Ok(models.try_convert()?)
    }

    async fn get_statistics(
        &self,
        alert_id: Option<i32>,
        user_id: Option<String>,
    ) -> Result<NotificationStatistics> {
        let start: u64 = (SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
            - Duration::from_secs(60 * 60 * 24).as_nanos())
        .try_into()?;

        #[derive(FromQueryResult)]
        struct DailyStatisticsQueryResult {
            total_response_time_sum: Option<f32>,
            num_responses_sum: Option<i64>,
            count: Option<i64>,
        }

        let daily_response_stats = {
            let mut query = db_entities::alert_notification::Entity::find()
                .filter(
                    db_entities::alert_notification::Column::CreatedAt
                        .into_expr()
                        .cast_as(Alias::new("bigint"))
                        .binary(BinOper::GreaterThanOrEqual, Expr::value(start)),
                )
                .select_only()
                .filter(db_entities::alert_notification::Column::DeletedAt.is_null())
                .left_join(db_entities::user_alert::Entity)
                .column_as(
                    db_entities::alert_notification::Column::TotalResponseTime.sum(),
                    "total_response_time_sum",
                )
                .column_as(
                    db_entities::alert_notification::Column::NumResponses.sum(),
                    "num_responses_sum",
                )
                .column_as(
                    db_entities::alert_notification::Column::AlertId.count(),
                    "count",
                );
            if let Some(alert_id) = alert_id {
                query = query.filter(db_entities::alert_notification::Column::AlertId.eq(alert_id));
            }
            if let Some(user_id) = &user_id {
                query = query.filter(db_entities::user_alert::Column::UserId.eq(user_id.clone()))
            }
            query
        }
        .into_model::<DailyStatisticsQueryResult>()
        .one(self)
        .await?
        .ok_or("No notifications found")?;

        #[derive(FromQueryResult)]
        struct TotalStatisticsQuery {
            count: Option<i64>,
        }
        let all_time_response_count = {
            let mut query = db_entities::alert_notification::Entity::find()
                .filter(db_entities::alert_notification::Column::DeletedAt.is_null())
                .left_join(db_entities::user_alert::Entity)
                .select_only()
                .column_as(
                    db_entities::alert_notification::Column::AlertId.count(),
                    "count",
                );
            if let Some(alert_id) = alert_id {
                query = query.filter(db_entities::alert_notification::Column::AlertId.eq(alert_id));
            }
            if let Some(user_id) = &user_id {
                query = query.filter(db_entities::user_alert::Column::UserId.eq(user_id.clone()))
            }
            query
        }
        .into_model::<TotalStatisticsQuery>()
        .one(self)
        .await?
        .ok_or("No notifications found")?
        .count;

        Ok(NotificationStatistics {
            total_alerts: all_time_response_count.unwrap_or_default().try_into()?,
            total_alerts_today: daily_response_stats.count.unwrap_or_default().try_into()?,
            avg_response_time: (daily_response_stats
                .total_response_time_sum
                .unwrap_or_default()
                / daily_response_stats.num_responses_sum.unwrap_or_default() as f32),
        })
    }

    async fn get_telegram_chat_id(&self, username: String) -> Result<String> {
        let row = db_entities::telegram_chat_id::Entity::find()
            .filter(db_entities::telegram_chat_id::Column::Username.eq(username))
            .one(self)
            .await?
            .ok_or("could not find chat id for username")?;

        Ok(row.chat_id)
    }

    async fn set_telegram_chat_id(&self, username: String, chat_id: String) -> Result<()> {
        if let Some(model) = db_entities::telegram_chat_id::Entity::find()
            .filter(db_entities::telegram_chat_id::Column::Username.eq(username.clone()))
            .one(self)
            .await?
        {
            let mut row = model.into_active_model();
            row.chat_id = Set(chat_id);
            row.update(self).await?;
        } else {
            let row = db_entities::telegram_chat_id::ActiveModel {
                username: Set(username.clone()),
                chat_id: Set(chat_id),
                ..Default::default()
            };
            row.insert(self).await?;
        };

        Ok(())
    }
    async fn get_endpoint(&self, client_id: &str) -> Result<String> {
        Ok(db_entities::jwt::Entity::find()
            .filter(db_entities::jwt::Column::ClientId.eq(client_id))
            .one(self)
            .await?
            .ok_or("Not found".to_owned())?
            .webhook_endpoint)
    }
}
