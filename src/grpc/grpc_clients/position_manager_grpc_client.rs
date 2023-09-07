// use std::{sync::Arc, time::Duration};

// use my_grpc_extensions::{GrpcChannel, GrpcClientSettings};
// use tonic::transport::Channel;


#[my_grpc_client_macros::generate_grpc_client(
    proto_file: "./proto/positions_manager_grpc_service.proto",
    crate_ns: "crate::position_manager_grpc",
    retries: 3,
    request_timeout_sec: 1,
    ping_timeout_sec: 1,
    ping_interval_sec: 3,
)]
pub struct PositionManagerGrpcClient {
    channel: my_grpc_extensions::GrpcChannel<TGrpcService>,
}

// use crate::{
//     position_manager_grpc::{
//         position_manager_grpc_service_client::PositionManagerGrpcServiceClient,
//         PositionManagerActivePositionGrpcModel, PositionManagerClosePositionGrpcRequest,
//         PositionManagerGetActivePositionGrpcRequest, PositionManagerGetActivePositionsGrpcRequest,
//         PositionManagerOpenPositionGrpcRequest, PositionManagerUpdateSlTpGrpcRequest,
//     },
//     trading_executor_grpc::{
//         TradingExecutorActivePositionGrpcModel, TradingExecutorClosedPositionGrpcModel,
//     },
//     TradingExecutorError,
// };

// struct PositionManagerSettingsGrpcUrl(String);

// impl PositionManagerSettingsGrpcUrl {
//     pub fn new(url: String) -> Self {
//         Self(url)
//     }
// }

// #[tonic::async_trait]
// impl GrpcClientSettings for PositionManagerSettingsGrpcUrl {
//     async fn get_grpc_url(&self, _: &'static str) -> String {
//         self.0.clone()
//     }
// }

// pub struct PositionManagerGrpcClient {
//     channel: GrpcChannel,
//     timeout: Duration,
// }

// impl PositionManagerGrpcClient {
//     pub async fn new(grpc_address: String) -> Self {
//         Self {
//             channel: GrpcChannel::new(
//                 Arc::new(PositionManagerSettingsGrpcUrl::new(grpc_address)),
//                 "postiion_manager",
//                 Duration::from_secs(10),
//             ),
//             timeout: Duration::from_secs(2),
//         }
//     }

//     async fn create_grpc_service(&self) -> PositionManagerGrpcServiceClient<Channel> {
//         return PositionManagerGrpcServiceClient::new(self.channel.get_channel().await.unwrap());
//     }

//     pub async fn open_position(
//         &self,
//         request: PositionManagerOpenPositionGrpcRequest,
//     ) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
//         let mut grpc_client = self.create_grpc_service().await;
//         println!("PM request: {:#?}", request);
//         let response = grpc_client
//             .open_position(request)
//             .await
//             .unwrap()
//             .into_inner();

//         println!("PM response: {:#?}", response);
//         if let Some(position) = response.position {
//             return Ok(position.into());
//         }

//         return Err(TradingExecutorError::from(response.status));
//     }

//     pub async fn close_position(
//         &self,
//         trader_id: &str,
//         account_id: &str,
//         position_id: &str,
//         process_id: &str,
//     ) -> Result<TradingExecutorClosedPositionGrpcModel, TradingExecutorError> {
//         let mut grpc_client = self.create_grpc_service().await;
//         let response = grpc_client
//             .close_position(PositionManagerClosePositionGrpcRequest {
//                 position_id: position_id.to_string(),
//                 process_id: process_id.to_string(),
//                 account_id: account_id.to_string(),
//                 trader_id: trader_id.to_string(),
//             })
//             .await
//             .unwrap()
//             .into_inner();

//         println!("PM response: {:#?}", response);
//         if let Some(position) = response.position {
//             return Ok(position.into());
//         }

//         return Err(TradingExecutorError::from(response.status));
//     }

//     pub async fn get_active_positions(
//         &self,
//         trader_id: &str,
//         account_id: &str,
//     ) -> Vec<PositionManagerActivePositionGrpcModel> {
//         let mut grpc_client = self.create_grpc_service().await;
//         let result = grpc_client
//             .get_account_active_positions(PositionManagerGetActivePositionsGrpcRequest {
//                 trader_id: trader_id.to_string(),
//                 account_id: account_id.to_string(),
//             })
//             .await
//             .unwrap();

//         return match my_grpc_extensions::read_grpc_stream::as_vec(result.into_inner(), self.timeout)
//             .await
//             .unwrap()
//         {
//             Some(result) => result,
//             None => vec![],
//         };
//     }

//     pub async fn get_active_position(
//         &self,
//         trader_id: &str,
//         account_id: &str,
//         position_id: &str,
//     ) -> Option<PositionManagerActivePositionGrpcModel> {
//         let request = PositionManagerGetActivePositionGrpcRequest {
//             trader_id: trader_id.to_string(),
//             account_id: account_id.to_string(),
//             position_id: position_id.to_string(),
//         };

//         println!("PM request: {:#?}", request);

//         let mut grpc_client = self.create_grpc_service().await;
//         let result = grpc_client
//             .get_active_position(request)
//             .await
//             .unwrap()
//             .into_inner();

//         println!("PM response: {:#?}", result);

//         return result.position;
//     }

//     pub async fn update_sl_tp(
//         &self,
//         request: PositionManagerUpdateSlTpGrpcRequest,
//     ) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
//         let mut grpc_client = self.create_grpc_service().await;

//         let response = grpc_client
//             .update_sl_tp(tonic::Request::new(request))
//             .await
//             .unwrap()
//             .into_inner();

//         if let Some(position) = response.position {
//             return Ok(position.into());
//         }

//         return Err(TradingExecutorError::from(response.status));
//     }
// }
