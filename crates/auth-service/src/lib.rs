use jsonwebtoken::{EncodingKey, Header};
use mempools_api::api::TokenMetadata;
use sea_orm::{DatabaseConnection, EntityTrait, Set, Unchanged};
use tonic::async_trait;
use util::service_registry::AuthServiceInterface;
use util::{JwtClaims, Result, ToGrpcResult};

#[derive(Clone)]
pub struct AuthService {
    db: DatabaseConnection,
    jwt_encoding_key: EncodingKey,
}

impl AuthService {
    pub fn new(db: DatabaseConnection, jwt_secret_key: &[u8]) -> Self {
        Self {
            db,
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret_key),
        }
    }
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn generate_jwt(&self, user_id: String, metadata: TokenMetadata) -> Result<String> {
        let new_token = jsonwebtoken::encode(
            &Header::default(),
            &JwtClaims {
                sub: user_id.clone(),
                exp: 0,
            },
            &self.jwt_encoding_key,
        )
        .to_grpc_result()?;

        db_entities::jwt::Entity::insert(db_entities::jwt::ActiveModel {
            jwt: Set(new_token.clone()),
            client_id: Set(user_id.clone()),
            name: Set(metadata.name),
            webhook_endpoint: Set(metadata.webhook_endpoint),
            valid: Set(true),
        })
        .exec(&self.db)
        .await?;
        Ok(new_token)
    }
    async fn set_jwt_status(&self, jwt: String, enabled: bool) -> Result<()> {
        db_entities::jwt::Entity::update(db_entities::jwt::ActiveModel {
            jwt: Unchanged(jwt),
            valid: Set(enabled),
            ..Default::default()
        })
        .exec(&self.db)
        .await
        .to_grpc_result()?;
        Ok(())
    }
}
