use chain_service::storage::ChainStorage;
use db_migration::ToDbResult;
use mempools_api::api::{CosmosChainData, CosmosEvmChainData, CreateChainRequest, EthChainData};
use sea_orm::DatabaseConnection;
use util::Result;

pub async fn add_test_data(db: &DatabaseConnection) -> Result<()> {
    let chains = vec![
        CreateChainRequest {
            name: "Archway Testnet".to_string(),
            icon: "".to_string(),
            chain_data: Some(mempools_api::api::ChainData {
                chain_data: Some(mempools_api::api::chain_data::ChainData::CosmosChainData(
                    CosmosChainData {
                        grpc_endpoint: "grpc.constantine.archway.tech:443".to_string(),
                        bech32_prefix: "arch".to_string(),
                    },
                )),
            }),
        },
        // CreateChainRequest {
        //     name: "Canto mainnet".to_string(),
        //     icon: "".to_string(),

        //     chain_data: Some(mempools_api::api::ChainData {
        //         chain_data: Some(
        //             mempools_api::api::chain_data::ChainData::CosmosEvmChainData(
        //                 CosmosEvmChainData {
        //                     cosmos_chain_data: Some(CosmosChainData {
        //                         grpc_endpoint: "https://canto.gravitychain.io:9090".to_string(),
        //                         bech32_prefix: "canto".to_string(),
        //                     }),
        //                     eth_chain_data: Some(EthChainData {
        //                         eth_rpc_endpoint: "https://canto.gravitychain.io".to_string(),
        //                     }),
        //                 },
        //             ),
        //         ),
        //     }),
        // },
        // CreateChainRequest {
        //     name: "Ethereum goerli testnet".to_string(),
        //     icon: "".to_string(),
        //     chain_data: Some(mempools_api::api::ChainData {
        //         chain_data: Some(mempools_api::api::chain_data::ChainData::EthChainData(
        //             EthChainData {
        //                 eth_rpc_endpoint: "https://rpc.ankr.com/eth_goerli".to_string(),
        //             },
        //         )),
        //     }),
        // },
    ];

    for chain in chains {
        db.create_chain(&chain).await.to_db_result()?;
    }

    Ok(())
}
