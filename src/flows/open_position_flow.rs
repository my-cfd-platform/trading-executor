use std::sync::Arc;

use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity,
    TradingProfileNoSqlEntity,
};

use crate::{
    position_manager_grpc::PositionManagerOpenPositionGrpcRequest,
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorOpenPositionGrpcRequest,
    },
    AppContext, TradingExecutorError,
};

pub async fn open_position(
    app: &Arc<AppContext>,
    request: TradingExecutorOpenPositionGrpcRequest,
) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
    // let datetime = Utc::now();

    let target_instrument = app
        .trading_instruments_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &request.asset_pair,
        )
        .await;

    let Some(_) = target_instrument else{
        return Err(TradingExecutorError::InstrumentNotFound)
    };
    let Some(target_account) = app
        .accounts_manager_grpc_client
        .get_client_account(&request.trader_id, &request.account_id)
        .await.account else{
            return Err(TradingExecutorError::AccountNotFound)
        };

    let Some(target_trading_group) = app.trading_groups_reader.get_entity(TradingGroupNoSqlEntity::generate_partition_key(), &target_account.trading_group).await else{
        return Err(TradingExecutorError::TradingGroupNotFound)
    };

    let Some(target_trading_profile) = app.trading_profiles_reader.get_entity(TradingProfileNoSqlEntity::generate_partition_key(), &target_trading_group.trading_profile_id).await else{
        return Err(TradingExecutorError::TradingProfileNotFound)
    };

    let Some(target_trading_profile_instrument) = target_trading_profile.instruments.iter().find(|x| x.id == request.asset_pair)else{
        return Err(TradingExecutorError::TradingProfileInstrumentNotFound)
    };

    if !target_trading_profile_instrument
        .leverages
        .contains(&request.leverage)
    {
        return Err(TradingExecutorError::MultiplierIsNotFound);
    }

    let open_position_request = PositionManagerOpenPositionGrpcRequest {
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
    };

    let position = app
        .position_manager_grpc_client
        .open_position(open_position_request)
        .await?;

    return Ok(position);
}
