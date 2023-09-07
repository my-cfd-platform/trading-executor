use std::sync::Arc;

use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity, TradingProfileNoSqlEntity,
};

use crate::{
    a_book_bridge_grpc::{ABookBridgeOpenPositionGrpcRequest, ABookBridgePositionSide},
    accounts_manager_grpc::{
        AccountManagerGetClientAccountGrpcRequest, AccountManagerUpdateAccountBalanceGrpcRequest,
        AccountsManagerOperationResult, UpdateBalanceReason,
    },
    position_manager_grpc::PositionManagerOpenPositionGrpcRequest,
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorOpenPositionGrpcRequest,
        TradingExecutorPositionSide,
    },
    AppContext, TradingExecutorError,
};

pub async fn open_position(
    app: &Arc<AppContext>,
    request: TradingExecutorOpenPositionGrpcRequest,
) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
    let position_id = uuid::Uuid::new_v4().to_string();

    let target_instrument = app
        .trading_instruments_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &request.asset_pair,
        )
        .await;

    let Some(target_instrument) = target_instrument else{
        return Err(TradingExecutorError::InstrumentNotFound)
    };
    let Some(target_account) = app
        .accounts_manager_grpc_client
        .get_client_account( AccountManagerGetClientAccountGrpcRequest { trader_id: request.trader_id.clone(), account_id: request.account_id.clone() }, &my_telemetry::MyTelemetryContext::new())
        .await.unwrap().account else{
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

    if target_trading_profile.is_a_book {
        let side: ABookBridgePositionSide = TradingExecutorPositionSide::from_i32(request.side)
            .unwrap()
            .into();
        let request = ABookBridgeOpenPositionGrpcRequest {
            instrument_id: request.asset_pair.to_string(),
            position_id: position_id.to_string(),
            account_id: request.account_id.to_string(),
            leverage: request.leverage as f64,
            invest_amount: request.invest_amount,
            side: side as i32,
        };

        let response = app
            .a_book_bridge_grpc_client
            .open_position(request, &my_telemetry::MyTelemetryContext::new())
            .await
            .unwrap();

        let result = {
            if response.status_code == 0 {
                Ok(response.position.unwrap())
            } else {
                Err(TradingExecutorError::ABookReject)
            }
        };
    }

    let balance_update_result = app
        .accounts_manager_grpc_client
        .update_client_account_balance(
            AccountManagerUpdateAccountBalanceGrpcRequest {
                trader_id: request.trader_id.clone(),
                account_id: request.account_id.clone(),
                delta: -request.invest_amount,
                comment: "Open position balance charge".to_string(),
                process_id: request.process_id.clone(),
                allow_negative_balance: false,
                reason: UpdateBalanceReason::TradingResult as i32,
                reference_transaction_id: None,
            },
            &my_telemetry::MyTelemetryContext::new(),
        )
        .await;

    if AccountsManagerOperationResult::Ok
        != AccountsManagerOperationResult::from_i32(balance_update_result.unwrap().result).unwrap()
    {
        return Err(TradingExecutorError::NotEnoughBalance);
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
        base: target_instrument.base.clone(),
        quote: target_instrument.quote.clone(),
        collateral_currency: "USD".to_string(),
        id: Some(position_id),
    };

    let position = app
        .position_manager_grpc_client
        .open_position(
            open_position_request,
            &my_telemetry::MyTelemetryContext::new(),
        )
        .await
        .unwrap();

    let position = position.position.unwrap().into();

    return Ok(position);
}
