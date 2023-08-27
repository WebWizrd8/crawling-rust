#![allow(clippy::await_holding_refcell_ref)]

use alert_service::AlertService;
use auth_service::AuthService;
use chain_service::ChainService;
use crawler_service::CrawlerService;
use db_migration::{Migrator, MigratorTrait};
use filter_service::FilterService;
use gateway_service::GatewayService;
use mempools_api::api::{gateway_admin_server::GatewayAdminServer, gateway_server::GatewayServer};
use notification_service::NotificationService;
use sea_orm::{ConnectOptions, Database};
use server::{config::Config, interceptors::AdminInterceptor};
use std::time::Duration;

use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::catch_panic::{CatchPanicLayer, ResponseForPanic};
use util::{
    service_registry::{RegistryServices, ServiceRegistry},
    Result,
};

#[macro_use]
extern crate log;

#[derive(Clone)]
struct PanicHandler;

impl ResponseForPanic for PanicHandler {
    type ResponseBody = String;

    fn response_for_panic(
        &mut self,
        err: Box<dyn std::any::Any + Send + 'static>,
    ) -> tonic::codegen::http::Response<Self::ResponseBody> {
        if let Some(s) = err.downcast_ref::<String>() {
            error!("Service panicked: {}", s);
        } else if let Some(s) = err.downcast_ref::<&str>() {
            error!("Service panicked: {}", s);
        } else {
            error!("Service panicked but `CatchPanic` was unable to downcast the panic info");
        };
        tonic::codegen::http::Response::new(format!("Service panicked: {:?}", err))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    let config = Config::from_env_or_default()?;
    let registry = ServiceRegistry::new();
    let db = Database::connect(
        ConnectOptions::new(config.application_config.db_url.clone())
            .sqlx_logging(false)
            .clone(),
    )
    .await?;

    // Set up database
    Migrator::up(&db, None).await?;
    #[cfg(feature = "dev")]
    server::test_data::add_test_data(&db).await?;

    let gateway_service = GatewayService::new(registry.clone());
    let crawler_service = CrawlerService::new(db.clone(), registry.clone());
    let filter_service = FilterService::new(registry.clone())?;
    let alert_service = AlertService::new(db.clone());
    let auth_service = AuthService::new(db.clone(), &config.application_config.jwt_secret);
    let chain_service = ChainService::new(db.clone());
    let notification_service = NotificationService::new(registry.clone(), db.clone())?;

    let svcs = RegistryServices {
        filter_service: Box::new(filter_service.clone()),
        alert_service: Box::new(alert_service.clone()),
        auth_service: Box::new(auth_service.clone()),
        notification_service: Box::new(notification_service.clone()),
        chain_service: Box::new(chain_service.clone()),
    };
    registry.register_services(svcs).await;

    // Daemons
    crawler_service.spawn_daemons();

    // Create server
    let server = Server::builder()
        .accept_http1(true)
        .timeout(Duration::from_secs(30));

    // Add Layers

    #[cfg(not(feature = "cors"))]
    let server = server.layer(tower_http::cors::CorsLayer::very_permissive().allow_origin(
        tower_http::cors::AllowOrigin::predicate(|origin, _| {
            origin.as_bytes().ends_with(b".mempools.com")
        }),
    ));

    #[cfg(feature = "cors")]
    let server = server.layer(tower_http::cors::CorsLayer::very_permissive());

    let server = server.layer(CatchPanicLayer::custom(PanicHandler));
    let mut server = server.layer(GrpcWebLayer::new());

    // Add Services

    #[cfg(feature = "reflection")]
    server.add_service(
        tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(mempools_api::FILE_DESCRIPTOR_SET)
            .build()?,
    );

    let auth_interceptor;
    #[cfg(feature = "auth")]
    {
        auth_interceptor =
            server::interceptors::AuthInterceptor::new(&config.application_config.jwt_secret)?;
    }
    #[cfg(not(feature = "auth"))]
    {
        auth_interceptor = server::interceptors::MockIdTokenSetter::new("1");
    }

    let gateway_server = tower::ServiceBuilder::new()
        .layer(tonic::service::interceptor(auth_interceptor.clone()))
        .service(GatewayServer::new(gateway_service.clone()));
    let gateway_admin_server = tower::ServiceBuilder::new()
        .layer(tonic::service::interceptor(AdminInterceptor::new(
            std::env::var("ADMIN_TOKEN").expect("Missing admin token"),
        )))
        .service(GatewayAdminServer::new(gateway_service.clone()));

    let server = server
        .add_service(gateway_server)
        .add_service(gateway_admin_server);

    let addr = "0.0.0.0:8123".parse()?;
    info!("started mempools-server on port 8123...");
    tokio::task::spawn(async move { server.serve(addr).await }).await??;

    Ok(())
}
