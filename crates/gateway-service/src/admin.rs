use crate::GatewayService;

use mempools_api::api::{
    gateway_admin_server::GatewayAdmin, CreateChainRequest, CreateChainResponse, GrantJwtRequest,
    GrantJwtResponse, UpdateChainRequest, UpdateChainResponse, UpdateJwtValidityRequest,
    UpdateJwtValidityResponse,
};

use request_validation::Validateable;
use tonic::{Request, Response, Status};
use util::ToGrpcResult;

#[tonic::async_trait]
impl GatewayAdmin for GatewayService {
    async fn create_chain(
        &self,
        request: Request<CreateChainRequest>,
    ) -> Result<Response<CreateChainResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let registry = self.registry.get_services().await.to_grpc_result()?;
        let chain_svc = registry.chain_service;
        Ok(Response::new(
            chain_svc
                .create_chain(request.get_ref())
                .await
                .to_grpc_result()?,
        ))
    }
    async fn update_chain(
        &self,
        request: Request<UpdateChainRequest>,
    ) -> Result<Response<UpdateChainResponse>, Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let registry = self.registry.get_services().await.to_grpc_result()?;
        let chain_svc = registry.chain_service;
        Ok(Response::new(
            chain_svc
                .update_chain(request.get_ref())
                .await
                .to_grpc_result()?,
        ))
    }
    async fn grant_jwt(
        &self,
        request: tonic::Request<GrantJwtRequest>,
    ) -> Result<tonic::Response<GrantJwtResponse>, tonic::Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let request = request.into_inner();
        let jwt = self
            .registry
            .get_services()
            .await
            .to_grpc_result()?
            .auth_service
            .generate_jwt(
                request.client_id.clone(),
                request
                    .metadata
                    .ok_or("Missing metadata")
                    .to_grpc_result()?,
            )
            .await
            .to_grpc_result()?;

        Ok(Response::new(GrantJwtResponse { jwt }))
    }
    async fn update_jwt_validity(
        &self,
        request: tonic::Request<UpdateJwtValidityRequest>,
    ) -> Result<tonic::Response<UpdateJwtValidityResponse>, tonic::Status> {
        request
            .validate(self.registry.clone())
            .await
            .to_grpc_result()?;

        let request = request.into_inner();

        self.registry
            .get_services()
            .await
            .to_grpc_result()?
            .auth_service
            .set_jwt_status(request.jwt, request.valid)
            .await
            .to_grpc_result()?;

        Ok(Response::new(UpdateJwtValidityResponse {}))
    }
}
