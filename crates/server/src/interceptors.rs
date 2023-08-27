use base64::Engine;
use jsonwebtoken::{DecodingKey, Validation};
use log::debug;
use sea_orm::{ConnectOptions, Database, EntityTrait};
use tokio::runtime::Builder;
use tonic::service::Interceptor;
use util::{JwtClaims, Result};
use util::{ToGrpcResult, UserMetadata};

#[derive(Clone)]
pub struct AdminInterceptor {
    admin_token: String,
}

impl AdminInterceptor {
    pub fn new(admin_token: String) -> Self {
        Self { admin_token }
    }

    pub fn is_admin(&self, bearer_token: String) -> bool {
        self.admin_token == bearer_token
    }
}

#[tonic::async_trait]
impl Interceptor for AdminInterceptor {
    fn call(
        &mut self,
        request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        let bearer_token = request
            .metadata()
            .get("Authorization")
            .ok_or("could not find Authorization header to validate admin token")
            .to_grpc_result()?
            .to_str()
            .to_grpc_result()?
            .split_once(' ')
            .ok_or("incorrect authorization header format - must be in bearer format")
            .to_grpc_result()?
            .1
            .to_owned();
        if !self.is_admin(bearer_token) {
            return Err(tonic::Status::permission_denied(
                "only admins can access this endpoint",
            ));
        }
        debug!("is_admin done");

        Ok(request)
    }
}

#[derive(Clone)]
pub struct AuthInterceptor {
    jwt_decoding_key: DecodingKey,
}

impl AuthInterceptor {
    pub fn new(secret_key: &[u8]) -> Result<Self> {
        Ok(Self {
            jwt_decoding_key: DecodingKey::from_secret(secret_key),
        })
    }
}

impl Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        let bearer_token = request
            .metadata()
            .get("Authorization")
            .ok_or("could not find Authorization header to validate jwt")
            .to_grpc_result()?
            .to_str()
            .to_grpc_result()?
            .split_once(' ')
            .ok_or("incorrect authorization header format - must be in bearer format")
            .to_grpc_result()?
            .1
            .to_owned();
        let mut validation = Validation::default();
        validation.validate_exp = false;
        let data =
            jsonwebtoken::decode::<JwtClaims>(&bearer_token, &self.jwt_decoding_key, &validation)
                .to_grpc_result()?
                .claims;
        let metadata: UserMetadata = UserMetadata {
            client_id: data.sub,
        };
        request.extensions_mut().insert(metadata);
        let db_result: Result<bool> = std::thread::spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .to_grpc_result()?
                .block_on(async move {
                    // Unsure why we have to connect to the DB again rather than just cloning the pool
                    let db = Database::connect(
                        ConnectOptions::new(std::env::var("db_url").unwrap())
                            .sqlx_logging(false)
                            .clone(),
                    )
                    .await
                    .unwrap();

                    Ok(db_entities::jwt::Entity::find_by_id(bearer_token)
                        .one(&db)
                        .await?
                        .ok_or("Jwt not found")?
                        .valid)
                })
        })
        .join()
        .map_err(|e| format!("{:?}", e))
        .to_grpc_result()?;
        if !db_result.to_grpc_result()? {
            return Err("Jwt is revoked").to_grpc_result();
        }

        Ok(request)
    }
}

#[derive(Clone)]
pub struct MockIdTokenSetter {
    default_user_id: String,
}

impl MockIdTokenSetter {
    pub fn new(default_user_id: impl ToString) -> Self {
        Self {
            default_user_id: default_user_id.to_string(),
        }
    }
}

impl Interceptor for MockIdTokenSetter {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> core::result::Result<tonic::Request<()>, tonic::Status> {
        let claims = UserMetadata {
            client_id: self.default_user_id.clone(),
        };

        if !request.metadata().contains_key("authorization") {
            let json_claims = serde_json::to_string(&claims).to_grpc_result()?;
            let b64_claims = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(json_claims);
            let mock_id_token = format!("mock_header.{}.mock_signature", b64_claims);

            // DONT MAKE THIS caps tonic bug - https://github.com/hyperium/tonic/issues/343
            request.metadata_mut().insert(
                "authorization",
                format!("Bearer {}", mock_id_token)
                    .as_str()
                    .parse()
                    .to_grpc_result()?,
            );
        }

        Ok(request)
    }
}
