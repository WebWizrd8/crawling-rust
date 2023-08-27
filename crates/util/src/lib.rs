use std::{error::Error, u8};

use cosmrs::bip32::secp256k1::sha2::Sha256;
use cosmrs::{bip32::secp256k1::sha2::Digest, tx::SignerInfo};

use serde::{Deserialize, Serialize};

use tonic::Status;
use web3::types::{H160, H256};
pub mod clients;
pub mod convert;
pub mod service_registry;

// pub const NOTIFIER_URL: &str = "https://firebase-notifier-eww3betigq-ue.a.run.app";

// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(tag = "t", content = "c")]
// pub enum NotifierData {
//     ValidityCheck,
//     Message(String),
// }
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct NotifierBody {
//     pub data: NotifierData,
//     pub registration_token: String,
// }

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
// pub type Result<T> = anyhow::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait ToGrpcResult<T> {
    fn to_grpc_result(self) -> core::result::Result<T, Status>;
}

impl<T, E: Into<Box<dyn Error + Send + Sync>>> ToGrpcResult<T> for core::result::Result<T, E> {
    fn to_grpc_result(self) -> core::result::Result<T, Status> {
        self.map_err(|err| {
            let err: Box<dyn Error> = err.into();
            Status::internal(format!("Unexpected error: {}", err))
        })
    }
}

pub trait ToResult<T> {
    fn to_result(self) -> Result<T>;
}

impl<T, E: ToString> ToResult<T> for core::result::Result<T, E> {
    fn to_result(self) -> Result<T> {
        self.map_err(|err| err.to_string().into())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMetadata {
    pub client_id: String,
}

pub trait GetMetadata {
    fn get_claims(&self) -> Result<UserMetadata>;
}

impl<T> GetMetadata for tonic::Request<T> {
    fn get_claims(&self) -> Result<UserMetadata> {
        Ok(self
            .extensions()
            .get::<UserMetadata>()
            .ok_or("metadata extension not set")?
            .clone())
    }
}

pub fn get_sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);

    hex::encode(hasher.finalize())
}

pub trait HashString {
    fn hash_string(&self) -> Result<String>;
}

impl HashString for H160 {
    fn hash_string(&self) -> Result<String> {
        Ok(format!("{:#x}", self))
    }
}

impl HashString for H256 {
    fn hash_string(&self) -> Result<String> {
        Ok(format!("{:#x}", self))
    }
}

pub fn get_signers_from_tx(chain_prefix: String, tx: cosmrs::Tx) -> Vec<String> {
    let mut all_signers = vec![];
    for info in tx.auth_info.signer_infos {
        if let Ok(signers) = signer_info_to_account_id(info, chain_prefix.clone()) {
            signers
                .iter()
                .for_each(|signer| all_signers.push(signer.clone()));
        }
    }

    all_signers
}

fn signer_info_to_account_id(info: SignerInfo, chain_prefix: String) -> Result<Vec<String>> {
    let temp = info
        .public_key
        .ok_or("could not get pub key for signer".to_string())?;
    let signers = match temp {
        cosmrs::tx::SignerPublicKey::Single(s) => {
            vec![s.account_id(&chain_prefix)?.to_string()]
        }
        cosmrs::tx::SignerPublicKey::LegacyAminoMultisig(s) => {
            let pks = s.public_keys;

            let mut signers = vec![];
            for pk in pks {
                signers.push(pk.account_id(&chain_prefix)?.to_string())
            }

            signers
        }
        cosmrs::tx::SignerPublicKey::Any(_a) => return Err("unexpected signer type".into()),
    };

    Ok(signers)
}

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
}
