use std::sync::Arc;

use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity, TradingProfileNoSqlEntity,
};
use service_sdk::my_telemetry::MyTelemetryContext;

use crate::{
    accounts_manager_grpc::AccountManagerGetClientAccountGrpcRequest,
    position_manager_grpc::{
        PositionManagerCancelPendingGrpcRequest, PositionManagerClosePositionGrpcRequest,
        PositionManagerOpenPendingGrpcRequest,
    },
    trading_executor_grpc::{
        TradingExecutorCancelPendingGrpcRequest, TradingExecutorOpenPendingGrpcRequest,
        TradingExecutorPendingPositionGrpcModel,
    },
    AppContext, TradingExecutorError,
};

pub async fn open_limit(
    app: &Arc<AppContext>,
    request: TradingExecutorOpenPendingGrpcRequest,
    telemetry_context: &MyTelemetryContext,
) -> Result<TradingExecutorPendingPositionGrpcModel, TradingExecutorError> {
    let position_id = uuid::Uuid::new_v4().to_string();

    let target_instrument = app
        .trading_instruments_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &request.asset_pair,
        )
        .await;

    let Some(target_instrument) = target_instrument else {
        return Err(TradingExecutorError::InstrumentNotFound);
    };
    let Some(target_account) = app
        .accounts_manager_grpc_client
        .get_client_account(
            AccountManagerGetClientAccountGrpcRequest {
                trader_id: request.trader_id.clone(),
                account_id: request.account_id.clone(),
            },
            telemetry_context,
        )
        .await
        .unwrap()
        .account
    else {
        return Err(TradingExecutorError::AccountNotFound);
    };

    let Some(target_trading_group) = app
        .trading_groups_reader
        .get_entity(
            TradingGroupNoSqlEntity::generate_partition_key(),
            &target_account.trading_group,
        )
        .await
    else {
        return Err(TradingExecutorError::TradingGroupNotFound);
    };

    let Some(target_trading_profile) = app
        .trading_profiles_reader
        .get_entity(
            TradingProfileNoSqlEntity::generate_partition_key(),
            &target_trading_group.trading_profile_id,
        )
        .await
    else {
        return Err(TradingExecutorError::TradingProfileNotFound);
    };

    let Some(target_trading_profile_instrument) = target_trading_profile
        .instruments
        .iter()
        .find(|x| x.id == request.asset_pair)
    else {
        return Err(TradingExecutorError::TradingProfileInstrumentNotFound);
    };

    if !target_trading_profile_instrument
        .leverages
        .contains(&request.leverage)
    {
        return Err(TradingExecutorError::MultiplierIsNotFound);
    }

    let open_position_request = PositionManagerOpenPendingGrpcRequest {
        asset_pair: request.asset_pair.clone(),
        side: request.side,
        invest_amount: request.invest_amount,
        leverage: request.leverage as f64,
        stop_out_percent: target_trading_profile.stop_out_percent,
        process_id: request.process_id.clone(),
        tp_in_profit: request.tp_in_profit,
        sl_in_profit: request.sl_in_profit,
        tp_in_asset_price: request.tp_in_asset_price,
        sl_in_asset_price: request.sl_in_asset_price,
        open_price: None,
        open_bid_ask: None,
        account_id: request.account_id.clone(),
        trader_id: request.trader_id.clone(),
        base: target_instrument.base.clone(),
        quote: target_instrument.quote.clone(),
        collateral_currency: "USD".to_string(),
        id: Some(position_id),
        desire_price: request.desire_price,
    };

    let position = app
        .position_manager_grpc_client
        .open_pending(open_position_request, telemetry_context)
        .await
        .unwrap();



    let position = position
        .position
        .ok_or(TradingExecutorError::PositionNotFound)?;

    let position = position.into();

    return Ok(position);
}

pub async fn cancel_pending(
    app: &Arc<AppContext>,
    request: TradingExecutorCancelPendingGrpcRequest,
    telemetry_context: &MyTelemetryContext,
) -> Result<TradingExecutorPendingPositionGrpcModel, TradingExecutorError> {
    let cancel_request = PositionManagerCancelPendingGrpcRequest {
        account_id: request.account_id,
        trader_id: request.trader_id,
        id: request.position_id,
    };

    let position = app
        .position_manager_grpc_client
        .cancel_pending(cancel_request, telemetry_context)
        .await
        .unwrap();

    let position = position
        .position
        .ok_or(TradingExecutorError::PositionNotFound)?;

    return Ok(position.into());
}
