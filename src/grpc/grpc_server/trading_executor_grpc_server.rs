use crate::{
    cancel_pending, close_position, open_limit, open_position,
    position_manager_grpc::{
        PositionManagerGetActivePositionsGrpcRequest, PositionManagerGetPendingPositionsGrpcRequest,
    },
    trading_executor_grpc::{
        trading_executor_grpc_service_server::TradingExecutorGrpcService,
        TradingExecutorActivePositionGrpcModel, TradingExecutorCancelPendingGrpcRequest,
        TradingExecutorCancelPendingPositionGrpcResponse, TradingExecutorClosePositionGrpcRequest,
        TradingExecutorClosePositionGrpcResponse,
        TradingExecutorGetAccountPendingPositionGrpcRequest,
        TradingExecutorGetActivePositionsGrpcRequest, TradingExecutorOpenPendingGrpcRequest,
        TradingExecutorOpenPendingPositionGrpcResponse, TradingExecutorOpenPositionGrpcRequest,
        TradingExecutorOpenPositionGrpcResponse, TradingExecutorOperationsCodes,
        TradingExecutorPendingPositionGrpcModel, TradingExecutorUpdateSlTpGrpcRequest,
        TradingExecutorUpdateSlTpGrpcResponse,
    },
    update_sl_tp, GrpcService,
};
use my_grpc_extensions::prelude::Stream;
use my_grpc_extensions::server::with_telemetry;
use service_sdk::my_grpc_extensions;
use std::pin::Pin;

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

    type GetAccountPendingPositionsStream = Pin<
        Box<
            dyn Stream<Item = Result<TradingExecutorPendingPositionGrpcModel, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    // #[with_telemetry]
    async fn open_position(
        &self,
        request: tonic::Request<TradingExecutorOpenPositionGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorOpenPositionGrpcResponse>, tonic::Status> {
        let request = request.into_inner();

        let my_telemetry = &service_sdk::my_telemetry::MyTelemetryContext::new();
        let open_position_result = open_position(&self.app, request, my_telemetry).await;

        let response = match open_position_result {
            Ok(position) => TradingExecutorOpenPositionGrpcResponse {
                status: TradingExecutorOperationsCodes::Ok as i32,
                position: Some(position),
            },
            Err(error) => {
                let error: TradingExecutorOperationsCodes = error.into();
                TradingExecutorOpenPositionGrpcResponse {
                    status: error.into(),
                    position: None,
                }
            }
        };

        Ok(tonic::Response::new(response))
    }

    #[with_telemetry]
    async fn close_position(
        &self,
        request: tonic::Request<TradingExecutorClosePositionGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorClosePositionGrpcResponse>, tonic::Status> {
        let request = request.into_inner();

        let open_position_result = close_position(&self.app, request, my_telemetry).await;

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

    #[with_telemetry]
    async fn set_pending_position(
        &self,
        request: tonic::Request<TradingExecutorOpenPendingGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorOpenPendingPositionGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();

        let result = open_limit(&self.app, request, my_telemetry).await;

        let response = match result {
            Ok(position) => TradingExecutorOpenPendingPositionGrpcResponse {
                status: 0,
                position: Some(position),
            },
            Err(error) => {
                let error: TradingExecutorOperationsCodes = error.into();
                TradingExecutorOpenPendingPositionGrpcResponse {
                    status: error.into(),
                    position: None,
                }
            }
        };
        Ok(tonic::Response::new(response))
    }

    #[with_telemetry]
    async fn cancel_pending_position(
        &self,
        request: tonic::Request<TradingExecutorCancelPendingGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorCancelPendingPositionGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let result = cancel_pending(&self.app, request, my_telemetry).await;

        let response = match result {
            Ok(position) => TradingExecutorCancelPendingPositionGrpcResponse {
                status: 0,
                position: Some(position),
            },
            Err(error) => {
                let error: TradingExecutorOperationsCodes = error.into();
                TradingExecutorCancelPendingPositionGrpcResponse {
                    status: error.into(),
                    position: None,
                }
            }
        };
        Ok(tonic::Response::new(response))
    }

    #[with_telemetry]
    async fn get_account_pending_positions(
        &self,
        request: tonic::Request<TradingExecutorGetAccountPendingPositionGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetAccountPendingPositionsStream>, tonic::Status> {
        let request = request.into_inner();

        let positions = self
            .app
            .position_manager_grpc_client
            .get_account_pending_positions(
                PositionManagerGetPendingPositionsGrpcRequest {
                    trader_id: request.trader_id,
                    account_id: request.account_id,
                },
                my_telemetry,
            )
            .await
            .unwrap();

        let positions = match positions {
            Some(src) => src,
            None => vec![],
        };

        my_grpc_extensions::grpc_server::send_vec_to_stream(positions.into_iter(), |x| x.into())
            .await
    }

    #[with_telemetry]
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
                &my_telemetry,
            )
            .await
            .unwrap();

        let positions = match positions {
            Some(src) => src,
            None => vec![],
        };

        my_grpc_extensions::grpc_server::send_vec_to_stream(positions.into_iter(), |x| x.into())
            .await
    }

    #[with_telemetry]
    async fn update_sl_tp(
        &self,
        request: tonic::Request<TradingExecutorUpdateSlTpGrpcRequest>,
    ) -> Result<tonic::Response<TradingExecutorUpdateSlTpGrpcResponse>, tonic::Status> {
        let request = request.into_inner();
        let update_sl_tp_result = update_sl_tp(&self.app, request, my_telemetry).await;

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

    async fn ping(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        return Ok(tonic::Response::new(()));
    }
}
