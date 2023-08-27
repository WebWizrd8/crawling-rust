use std::{sync::Arc, time::Duration};

use cosmrs::{
    proto::cosmos::{
        base::abci::v1beta1::{AbciMessageLog, TxResponse},
        tx::v1beta1::Tx,
    },
    Any,
};
use dyn_clone::DynClone;
use mempools_api::api::{
    alert_notification_data::AlertNotificationData, AlertSource, BackendUserAlert, CosmosChainData,
    CreateAlertRequest, CreateChainRequest, CreateChainResponse, EthChainData, GetChainsResponse,
    TokenMetadata, UpdateChainRequest, UpdateChainResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::Result;

#[derive(Clone)]
pub struct RegistryServices {
    pub filter_service: Box<dyn FilterServiceInterface>,
    pub alert_service: Box<dyn AlertServiceInterface>,
    pub auth_service: Box<dyn AuthServiceInterface>,
    pub notification_service: Box<dyn NotificationServiceInterface>,
    pub chain_service: Box<dyn ChainServiceInterface>,
}

#[tonic::async_trait]
pub trait FilterServiceInterface: DynClone + Send + Sync + 'static {
    async fn process_alert_source(&self, alert_source: ProcessAlertSourceRequeust) -> Result<()>;
    async fn process_alert_for_alert_source(
        &self,
        alert_source: ProcessAlertSourceRequeust,
        alert: BackendUserAlert,
    ) -> Result<()>;
}

#[tonic::async_trait]
pub trait AlertServiceInterface: DynClone + Send + Sync + 'static {
    async fn get_alert_by_id(&self, alert_id: String) -> Result<BackendUserAlert>;
    async fn get_alerts(
        &self,
        filter: AlertFilter,
        page: Option<u64>,
    ) -> Result<Vec<BackendUserAlert>>;
    async fn create_alert(
        &self,
        req: &CreateAlertRequest,
        client_id: String,
    ) -> Result<BackendUserAlert>;
    async fn update_alert(&self, alert: BackendUserAlert) -> Result<BackendUserAlert>;
    async fn delete_alert(&self, id: i32) -> Result<()>;
}
#[tonic::async_trait]
pub trait AuthServiceInterface: DynClone + Send + Sync + 'static {
    async fn generate_jwt(&self, user_id: String, metadata: TokenMetadata) -> Result<String>;
    async fn set_jwt_status(&self, jwt: String, enabled: bool) -> Result<()>;
}

#[tonic::async_trait]
pub trait ChainServiceInterface: DynClone + Send + Sync + 'static {
    async fn get_chains(&self) -> Result<GetChainsResponse>;
    async fn create_chain(&self, req: &CreateChainRequest) -> Result<CreateChainResponse>;
    async fn update_chain(&self, req: &UpdateChainRequest) -> Result<UpdateChainResponse>;
}

#[tonic::async_trait]
pub trait NotificationServiceInterface: DynClone + Send + Sync + 'static {
    async fn get_notifications(
        &self,
        filter: NotificationFilter,
        page: u64,
    ) -> Result<Vec<mempools_api::api::AlertNotification>>;
    async fn send_notification(
        &self,
        notification: Notification,
        alert_owner_jwt: String,
    ) -> Result<()>;
    async fn get_statistics(
        &self,
        alert_id: Option<i32>,
        user_id: Option<String>,
    ) -> Result<NotificationStatistics>;
}

dyn_clone::clone_trait_object!(FilterServiceInterface);
dyn_clone::clone_trait_object!(AlertServiceInterface);
dyn_clone::clone_trait_object!(AuthServiceInterface);
dyn_clone::clone_trait_object!(NotificationServiceInterface);
dyn_clone::clone_trait_object!(ChainServiceInterface);

#[derive(Clone, Default)]
pub struct ServiceRegistry(Arc<RwLock<Option<RegistryServices>>>);

impl ServiceRegistry {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }

    pub async fn register_services(&self, svcs: RegistryServices) {
        *self.0.write().await = Some(svcs)
    }

    pub async fn get_services(&self) -> Result<RegistryServices> {
        Ok(self
            .0
            .read()
            .await
            .as_ref()
            .ok_or("could not find registry")?
            .clone())
    }
}

#[derive(Clone, Default)]
pub struct AlertFilter {
    pub id: Option<i32>,
    pub user_id: Option<String>,
    pub chain_id: Option<i32>,
    pub alert_source: Option<AlertSource>,
}

#[derive(Clone, Default)]
pub struct NotificationFilter {
    pub user_id: String,
    pub id: Option<i32>,
    pub alert_id: Option<i32>,
    pub time: Option<TimeRange>,
}

#[derive(Clone, Default)]
pub struct TimeRange {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

pub enum Notification {
    AlertNotification(AlertNotification),
}

pub struct NotificationStatistics {
    pub total_alerts: u32,
    pub total_alerts_today: u32,
    pub avg_response_time: f32,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlertNotification {
    pub notification: AlertNotificationData,
    pub alert_id: String,
    pub alert_source_id: String,
}

pub struct ResponseTime {
    pub total_response_time: Duration,
    pub num_responses: u64,
}

#[derive(Clone)]
pub enum ProcessAlertSourceRequeust {
    CosmosTx(Box<AlertSourceCosmosTx>),
    CosmosMsg(Box<AlertSourceCosmosMsg>),
    EthLog(Box<AlertSourceEthLog>),
    EthTx(Box<AlertSourceEthTx>),
    ArchwaysBroadcast {
        chain_id: String,
        message: String,
        client_id: String,
    },
}

#[derive(Clone)]
pub struct AlertSourceCosmosTx {
    pub chain_id: String,
    pub chain_data: CosmosChainData,
    pub tx: Tx,
    pub tx_resp: TxResponse,
    pub tx_hash: String,
}
#[derive(Clone)]
pub struct AlertSourceCosmosMsg {
    pub chain_id: String,
    pub chain_data: CosmosChainData,
    pub msg_log: Option<AbciMessageLog>,
    pub msg_index: u64,
    pub msg: Any,
    pub tx_hash: String,
}

#[derive(Clone)]
pub struct AlertSourceEthLog {
    pub chain_id: String,
    pub chain_data: EthChainData,
    pub tx_hash: String,
    pub log: web3::types::Log,
    pub log_index: u64,
}

#[derive(Clone)]
pub struct AlertSourceEthTx {
    pub chain_id: String,
    pub chain_data: EthChainData,
    pub tx_hash: String,
    pub tx: web3::types::Transaction,
    pub tx_resp: web3::types::TransactionReceipt,
}

#[derive(Clone)]
pub struct AlertSourceContext {
    pub id: String,
    pub chain_id: String,
    pub source_type: AlertSource,
}

impl ProcessAlertSourceRequeust {
    pub fn ctx(&self) -> AlertSourceContext {
        match self.clone() {
            ProcessAlertSourceRequeust::CosmosTx(tx) => AlertSourceContext {
                id: tx.tx_hash,
                chain_id: tx.chain_id,
                source_type: AlertSource::CosmosTx,
            },
            ProcessAlertSourceRequeust::CosmosMsg(msg) => AlertSourceContext {
                id: msg.tx_hash,
                chain_id: msg.chain_id,
                source_type: AlertSource::CosmosMsg,
            },
            ProcessAlertSourceRequeust::EthLog(log) => AlertSourceContext {
                id: log.tx_hash,
                chain_id: log.chain_id,
                source_type: AlertSource::EthLog,
            },
            ProcessAlertSourceRequeust::EthTx(tx) => AlertSourceContext {
                id: tx.tx_hash,
                chain_id: tx.chain_id,
                source_type: AlertSource::EthTx,
            },
            ProcessAlertSourceRequeust::ArchwaysBroadcast { chain_id, .. } => AlertSourceContext {
                id: "".into(),
                chain_id,
                source_type: AlertSource::ArchwaysBroadcast,
            },
        }
    }

    pub fn get_cosmos_msg(&self) -> Result<AlertSourceCosmosMsg> {
        if let Self::CosmosMsg(msg) = self {
            Ok(*msg.clone())
        } else {
            Err("alert source mistmatch".into())
        }
    }

    pub fn get_cosmos_tx(&self) -> Result<AlertSourceCosmosTx> {
        if let Self::CosmosTx(tx) = self {
            Ok(*tx.clone())
        } else {
            Err("alert source mistmatch".into())
        }
    }

    pub fn get_eth_log(&self) -> Result<AlertSourceEthLog> {
        if let Self::EthLog(log) = self {
            Ok(*log.clone())
        } else {
            Err("alert source mistmatch".into())
        }
    }

    pub fn get_eth_tx(&self) -> Result<AlertSourceEthTx> {
        if let Self::EthTx(tx) = self {
            Ok(*tx.clone())
        } else {
            Err("alert source mistmatch".into())
        }
    }
}
