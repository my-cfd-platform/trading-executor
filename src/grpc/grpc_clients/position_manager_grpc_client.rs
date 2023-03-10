use std::{sync::Arc, time::Duration};

use my_grpc_extensions::{GrpcChannel, GrpcClientSettings};
use tonic::transport::Channel;

use crate::{
    position_manager_grpc::{
        position_manager_grpc_service_client::PositionManagerGrpcServiceClient,
        PositionManagerOpenPositionGrpcRequest, PositionManagerUpdateSlTpGrpcRequest,
        PositionManagerUpdateSlTpGrpcResponse,
    },
    trading_executor_grpc::TradingExecutorActivePositionGrpcModel,
    TradingExecutorError,
};

struct PositionManagerSettingsGrpcUrl(String);

impl PositionManagerSettingsGrpcUrl {
    pub fn new(url: String) -> Self {
        Self(url)
    }
}

#[tonic::async_trait]
impl GrpcClientSettings for PositionManagerSettingsGrpcUrl {
    async fn get_grpc_url(&self, _: &'static str) -> String {
        self.0.clone()
    }
}

pub struct PositionManagerGrpcClient {
    channel: GrpcChannel,
    timeout: Duration,
}

impl PositionManagerGrpcClient {
    pub async fn new(grpc_address: String) -> Self {
        Self {
            channel: GrpcChannel::new(
                Arc::new(PositionManagerSettingsGrpcUrl::new(grpc_address)),
                "postiion_manager",
                Duration::from_secs(10),
            ),
            timeout: Duration::from_secs(2),
        }
    }

    async fn create_grpc_service(&self) -> PositionManagerGrpcServiceClient<Channel> {
        return PositionManagerGrpcServiceClient::new(self.channel.get_channel().await.unwrap());
    }

    pub async fn open_position(
        &self,
        request: PositionManagerOpenPositionGrpcRequest,
    ) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
        let mut grpc_client = self.create_grpc_service().await;

        let response = grpc_client
            .open_position(tonic::Request::new(request))
            .await
            .unwrap()
            .into_inner();

        if let Some(position) = response.positon {
            return Ok(position.into());
        }

        return Err(TradingExecutorError::from(response.status));
    }

    pub async fn update_sl_tp(
        &self,
        request: PositionManagerUpdateSlTpGrpcRequest,
    ) -> Result<tonic::Response<PositionManagerUpdateSlTpGrpcResponse>, tonic::Status> {
        let mut grpc_client = self.create_grpc_service().await;

        return grpc_client.update_sl_tp(tonic::Request::new(request)).await;
    }
}
