use mempools_api::api::{
    alert::ChainAlert, cosmos_evm_alert, Alert, CosmosAlert, CosmosEvmAlert, CreateAlertRequest,
    CreateChainRequest, DeleteAlertRequest, EthAlert, GetAlertsRequest, GetChainsRequest,
    GetNotificationsRequest, GetStatisticsRequest, GrantJwtRequest, SendBroadcastRequest,
    UpdateAlertRequest, UpdateChainRequest, UpdateJwtValidityRequest,
};
use tonic::Request;
use util::{service_registry::ServiceRegistry, Result};

#[tonic::async_trait]
pub trait Validateable {
    async fn validate(&self, registry: ServiceRegistry) -> Result<()>;
}

#[tonic::async_trait]
impl Validateable for Request<CreateAlertRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        let req = self.get_ref();
        if req.chain_id.is_empty() {
            return Err("alert needs to specify chain".into());
        }

        if let Some(Alert {
            chain_alert: Some(chain_alert),
            ..
        }) = &req.alert
        {
            match chain_alert{
                ChainAlert::CosmosAlert(CosmosAlert{cosmos_alert: Some(cosmos_alert)}) => {
                    match cosmos_alert{
                        mempools_api::api::cosmos_alert::CosmosAlert::AlertCosmosSendFunds(a) => {
                            if a.from.is_empty() || a.to.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        mempools_api::api::cosmos_alert::CosmosAlert::AlertCosmosMonitorFunds(a) => {
                            if a.address.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        mempools_api::api::cosmos_alert::CosmosAlert::AlertCosmosSmartContractEvents(a) => {
                            if a.address.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        mempools_api::api::cosmos_alert::CosmosAlert::AlertCosmosTxOutcome(a) => {
                            if a.signer.is_empty(){
                                return Err("Signer must be specified".into())
                            }
                        },
                    }
                },
                ChainAlert::CosmosEvmAlert(CosmosEvmAlert{
                    cosmos_evm_alert: Some(cosmos_evm_alert)
                }) => {
                    match cosmos_evm_alert{
                        cosmos_evm_alert::CosmosEvmAlert::AlertEthMonitorFunds(a) => {
                            if a.address.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        cosmos_evm_alert::CosmosEvmAlert::AlertEthTxOutcome(a) => {
                            if a.signer.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        cosmos_evm_alert::CosmosEvmAlert::AlertEthSmartContractEvents(a) => {
                            if a.contract_addr.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        cosmos_evm_alert::CosmosEvmAlert::AlertCosmosMonitorFunds(a) => {
                            if a.address.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        cosmos_evm_alert::CosmosEvmAlert::AlertCosmosTxOutcome(a) => {
                            if a.signer.is_empty(){
                                return Err("Signer must be specified".into())
                            }
                        },
                    }
                },
                ChainAlert::EthAlert(EthAlert{ eth_alert: Some(eth_alert)}) => {
                    match eth_alert{
                        mempools_api::api::eth_alert::EthAlert::AlertEthMonitorFunds(a) => {
                            if a.address.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                        mempools_api::api::eth_alert::EthAlert::AlertEthTxOutcome(a) => {
                            if a.signer.is_empty(){
                                return Err("Signer must be specified".into())
                            }
                        },
                        mempools_api::api::eth_alert::EthAlert::AlertEthSmartContractEvents(a) => {
                            if a.contract_addr.is_empty(){
                                return Err("Address must be specified".into())
                            }
                        },
                    }
                },
                ChainAlert::ArchwayBroadcastAlert(_) => {},
                _ => {}
            }
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<GetAlertsRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<UpdateAlertRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<DeleteAlertRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<GetNotificationsRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<GetStatisticsRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<GetChainsRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<GrantJwtRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}
#[tonic::async_trait]
impl Validateable for Request<UpdateJwtValidityRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<CreateChainRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl Validateable for Request<UpdateChainRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}
#[tonic::async_trait]
impl Validateable for Request<SendBroadcastRequest> {
    async fn validate(&self, _registry: ServiceRegistry) -> Result<()> {
        Ok(())
    }
}
