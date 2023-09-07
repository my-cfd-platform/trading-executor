use std::pin::Pin;

use crate::{
    close_position, open_position,
    position_manager_grpc::{
        PositionManagerGetActivePositionGrpcRequest, PositionManagerGetActivePositionsGrpcRequest,
    },
    trading_executor_grpc::{
        trading_executor_grpc_service_server::TradingExecutorGrpcService,
        TradingExecutorActivePositionGrpcModel, TradingExecutorClosePositionGrpcRequest,
        TradingExecutorClosePositionGrpcResponse, TradingExecutorGetActivePositionsGrpcRequest,
        TradingExecutorOpenPositionGrpcRequest, TradingExecutorOpenPositionGrpcResponse,
        TradingExecutorOperationsCodes, TradingExecutorUpdateSlTpGrpcRequest,
        TradingExecutorUpdateSlTpGrpcResponse,
    },
    update_sl_tp, GrpcService,
};
use futures_core::Stream;

#[tonic::async_trait]
impl TradingExecutorGrpcService for GrpcService {
    type GetAccountActivePositionsStream = Pin<
        Box<
            dyn Stream<Item = Result<TradingExecutorActivePositionGrpcModel, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn open_position(
        &self,
        request: tonic::Request<TradingExecutorOpenPositionGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorOpenPositionGrpcResponse>, tonic::Status> {
        let request = request.into_inner();

        let open_position_result = open_position(&self.app, request).await;

        let response = match open_position_result {
            Ok(position) => TradingExecutorOpenPositionGrpcResponse {
                status: 0,
                positon: Some(position),
            },
            Err(error) => {
                let error: TradingExecutorOperationsCodes = error.into();
                TradingExecutorOpenPositionGrpcResponse {
                    status: error.into(),
                    positon: None,
                }
            }
        };

        Ok(tonic::Response::new(response))
    }

    async fn close_position(
        &self,
        request: tonic::Request<TradingExecutorClosePositionGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorClosePositionGrpcResponse>, tonic::Status> {
        let request = request.into_inner();

        let open_position_result = close_position(&self.app, request).await;

        let response = match open_position_result {
            Ok(position) => TradingExecutorClosePositionGrpcResponse {
                status: 0,
                position: Some(position),
            },
            Err(error) => {
                let error: TradingExecutorOperationsCodes = error.into();
                TradingExecutorClosePositionGrpcResponse {
                    status: error.into(),
                    position: None,
                }
            }
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_account_active_positions(
        &self,
        request: tonic::Request<TradingExecutorGetActivePositionsGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetAccountActivePositionsStream>, tonic::Status> {
        let request = request.into_inner();
        let positions = self
            .app
            .position_manager_grpc_client
            .get_account_active_positions(
                PositionManagerGetActivePositionsGrpcRequest {
                    trader_id: request.trader_id.clone(),
                    account_id: request.account_id.clone(),
                },
                &my_telemetry::MyTelemetryContext::new(),
            )
            .await
            .unwrap();

        let positions = match positions {
            Some(src) => src,
            None => vec![],
        };

        my_grpc_extensions::grpc_server::send_vec_to_stream(positions, |x| x.into()).await
    }

    async fn update_sl_tp(
        &self,
        request: tonic::Request<TradingExecutorUpdateSlTpGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorUpdateSlTpGrpcResponse>, tonic::Status> {
        let update_sl_tp_result = update_sl_tp(&self.app, request.into_inner()).await;

        let response: TradingExecutorUpdateSlTpGrpcResponse = match update_sl_tp_result {
            Ok(position) => TradingExecutorUpdateSlTpGrpcResponse {
                status: 0,
                position: Some(position),
            },
            Err(error) => {
                let error: TradingExecutorOperationsCodes = error.into();
                TradingExecutorUpdateSlTpGrpcResponse {
                    status: error.into(),
                    position: None,
                }
            }
        };

        Ok(tonic::Response::new(response))
    }

    async fn ping(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        return Ok(tonic::Response::new(()));
    }
}
