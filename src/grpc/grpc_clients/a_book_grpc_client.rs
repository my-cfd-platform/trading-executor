use std::{sync::Arc, time::Duration};

use my_grpc_extensions::{GrpcChannel, GrpcClientSettings};
use tonic::transport::Channel;

use crate::{
    a_book_bridge_grpc::{
        a_book_bridge_grpc_service_client::ABookBridgeGrpcServiceClient,
        ABookBridgeOpenPositionGrpcRequest, ABookBridgeOpenPositionGrpcResponsePositionModel,
        ABookBridgePositionSide,
    },
    trading_executor_grpc::TradingExecutorPositionSide, TradingExecutorError,
};

struct ABookBridgeSettingsGrpcUrl(String);

impl ABookBridgeSettingsGrpcUrl {
    pub fn new(url: String) -> Self {
        Self(url)
    }
}

#[tonic::async_trait]
impl GrpcClientSettings for ABookBridgeSettingsGrpcUrl {
    async fn get_grpc_url(&self, _: &'static str) -> String {
        self.0.clone()
    }
}

pub struct ABookBridgeGrpcClient {
    channel: GrpcChannel,
}

impl ABookBridgeGrpcClient {
    pub async fn new(grpc_address: String) -> Self {
        Self {
            channel: GrpcChannel::new(
                Arc::new(ABookBridgeSettingsGrpcUrl::new(grpc_address)),
                "a_book",
                Duration::from_secs(10),
            ),
        }
    }

    async fn create_grpc_service(&self) -> ABookBridgeGrpcServiceClient<Channel> {
        return ABookBridgeGrpcServiceClient::new(self.channel.get_channel().await.unwrap());
    }

    pub async fn open_position(
        &self,
        position_id: &str,
        account_id: &str,
        leverage: f64,
        invest_amount: f64,
        instrument: &str,
        side: TradingExecutorPositionSide,
    ) -> Result<ABookBridgeOpenPositionGrpcResponsePositionModel, TradingExecutorError> {
        let mut grpc_client = self.create_grpc_service().await;
        let side: ABookBridgePositionSide = side.into();
        let request = ABookBridgeOpenPositionGrpcRequest {
            instrument_id: instrument.to_string(),
            position_id: position_id.to_string(),
            account_id: account_id.to_string(),
            leverage,
            invest_amount,
            side: side as i32,
        };

        let response = grpc_client
            .open_position(tonic::Request::new(request))
            .await
            .unwrap()
            .into_inner();

        if response.status_code == 0 {
            Ok(response.position.unwrap())
        } else {
            Err(TradingExecutorError::ABookReject)
        }
    }
}
