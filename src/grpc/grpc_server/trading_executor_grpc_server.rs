use std::pin::Pin;

use crate::{
    map_error_to_grpc_status, open_position,
    trading_executor_grpc::{
        trading_executor_grpc_service_server::TradingExecutorGrpcService,
        TradingExecutorActivePositionGrpcModel, TradingExecutorClosePositionGrpcRequest,
        TradingExecutorClosePositionGrpcResponse, TradingExecutorGetActivePositionsGrpcRequest,
        TradingExecutorOpenPositionGrpcRequest, TradingExecutorOpenPositionGrpcResponse,
        TradingExecutorUpdateSlTpGrpcRequest, TradingExecutorUpdateSlTpGrpcResponse,
    },
    GrpcService,
};

#[tonic::async_trait]
impl TradingExecutorGrpcService for GrpcService {
    type GetAccountActivePositionsStream = Pin<
        Box<
            dyn tonic::codegen::futures_core::Stream<
                    Item = Result<TradingExecutorActivePositionGrpcModel, tonic::Status>,
                > + Send
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
            Err(error) => TradingExecutorOpenPositionGrpcResponse {
                status: map_error_to_grpc_status(&error),
                positon: None,
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn close_position(
        &self,
        request: tonic::Request<TradingExecutorClosePositionGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorClosePositionGrpcResponse>, tonic::Status> {
        todo!()
    }

    async fn get_account_active_positions(
        &self,
        request: tonic::Request<TradingExecutorGetActivePositionsGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetAccountActivePositionsStream>, tonic::Status> {
        todo!()
    }

    async fn update_sl_tp(
        &self,
        request: tonic::Request<TradingExecutorUpdateSlTpGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorUpdateSlTpGrpcResponse>, tonic::Status> {
        todo!()
    }
}
