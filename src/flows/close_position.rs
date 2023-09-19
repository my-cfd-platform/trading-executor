use std::sync::Arc;

use my_nosql_contracts::TradingInstrumentNoSqlEntity;

use crate::{
    position_manager_grpc::{
        PositionManagerClosePositionGrpcRequest, PositionManagerGetActivePositionGrpcRequest,
    },
    trading_executor_grpc::{
        TradingExecutorClosePositionGrpcRequest, TradingExecutorClosedPositionGrpcModel,
    },
    AppContext, TradingExecutorError,
};
use service_sdk::my_telemetry;

pub async fn close_position(
    app: &Arc<AppContext>,
    request: TradingExecutorClosePositionGrpcRequest,
    telemetry_context: &my_telemetry::MyTelemetryContext,
) -> Result<TradingExecutorClosedPositionGrpcModel, TradingExecutorError> {
    let Some(_) = app
        .accounts_manager_grpc_client
        .get_client_account(
            crate::accounts_manager_grpc::AccountManagerGetClientAccountGrpcRequest {
                trader_id: request.trader_id.clone(),
                account_id: request.account_id.clone(),
            },
            &telemetry_context,
        )
        .await
        .unwrap()
        .account
    else {
        return Err(TradingExecutorError::AccountNotFound);
    };

    let Some(target_position) = app
        .position_manager_grpc_client
        .get_active_position(
            PositionManagerGetActivePositionGrpcRequest {
                trader_id: request.trader_id.clone(),
                account_id: request.account_id.clone(),
                position_id: request.position_id.clone(),
            },
            telemetry_context,
        )
        .await
        .unwrap()
        .position
    else {
        return Err(TradingExecutorError::PositionNotFound);
    };

    let target_instrument = app
        .trading_instruments_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &target_position.asset_pair,
        )
        .await;

    let Some(_) = target_instrument else {
        return Err(TradingExecutorError::InstrumentNotFound);
    };

    let close_result = app
        .position_manager_grpc_client
        .close_position(
            PositionManagerClosePositionGrpcRequest {
                position_id: request.position_id,
                process_id: request.process_id,
                account_id: request.account_id,
                trader_id: request.trader_id,
            },
            telemetry_context,
        )
        .await;

    let close_result = close_result.unwrap();
    if let Some(position) = close_result.position {
        return Ok(position.into());
    };

    return Err(TradingExecutorError::from(close_result.status));
}
