use std::sync::Arc;

use crate::{
    accounts_manager_grpc::AccountManagerGetClientAccountGrpcRequest,
    position_manager_grpc::PositionManagerUpdateSlTpGrpcRequest,
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorUpdateSlTpGrpcRequest,
    },
    AppContext, TradingExecutorError,
};
use my_nosql_contracts::{TradingGroupNoSqlEntity, TradingProfileNoSqlEntity};
use service_sdk::my_telemetry;

pub async fn update_sl_tp(
    app: &Arc<AppContext>,
    request: TradingExecutorUpdateSlTpGrpcRequest,
    telemetry: &my_telemetry::MyTelemetryContext,
) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
    let Some(target_account) = app
        .accounts_manager_grpc_client
        .get_client_account(
            AccountManagerGetClientAccountGrpcRequest {
                trader_id: request.trader_id.clone(),
                account_id: request.account_id.clone(),
            },
            telemetry,
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

    let Some(_) = app
        .trading_profiles_reader
        .get_entity(
            TradingProfileNoSqlEntity::generate_partition_key(),
            &target_trading_group.trading_profile_id,
        )
        .await
    else {
        return Err(TradingExecutorError::TradingProfileNotFound);
    };

    let pm_request = PositionManagerUpdateSlTpGrpcRequest {
        position_id: request.position_id,
        account_id: request.account_id,
        trader_id: request.trader_id,
        tp_in_profit: request.tp_in_profit,
        tp_in_asset_price: request.tp_in_asset_price,
        sl_in_asset_price: request.sl_in_asset_price,
        sl_in_profit: request.sl_in_profit,
        process_id: request.process_id,
    };

    let update_result = app
        .position_manager_grpc_client
        .update_sl_tp(pm_request, &my_telemetry::MyTelemetryContext::new())
        .await;

    let update_result = update_result.unwrap();

    if let Some(position) = update_result.position {
        return Ok(position.into());
    }

    return Err(TradingExecutorError::from(update_result.status));
}
