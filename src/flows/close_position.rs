use std::sync::Arc;

use my_nosql_contracts::TradingInstrumentNoSqlEntity;

use crate::{
    position_manager_grpc::{
        PositionManagerClosePositionGrpcRequest, PositionManagerGetActivePositionGrpcRequest,
    },
    trading_executor_grpc::{
        TradingExecutorClosePositionGrpcRequest, TradingExecutorClosedPositionGrpcModel,
    },
    validate_instrument_day_off, validate_timeout, AppContext, TradingExecutorError,
};
use service_sdk::my_telemetry;

pub async fn close_position(
    app: &Arc<AppContext>,
    request: TradingExecutorClosePositionGrpcRequest,
    telemetry_context: &my_telemetry::MyTelemetryContext,
) -> Result<TradingExecutorClosedPositionGrpcModel, TradingExecutorError> {
    let Some(account) = app
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

    let Some(target_instrument) = target_instrument else {
        return Err(TradingExecutorError::InstrumentNotFound);
    };

    let position_to_close = app
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
        .unwrap();

    let Some(_position_to_close) = position_to_close.position else {
        return Err(TradingExecutorError::PositionNotFound);
    };

    validate_instrument_day_off(&target_instrument)?;
    validate_timeout(
        app,
        &target_instrument,
        &target_instrument.base,
        &target_instrument.quote,
        &account.currency,
    )
    .await?;

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
