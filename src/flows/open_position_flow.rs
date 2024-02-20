use core::panic;
use std::{sync::Arc, time::Duration};

use rand::Rng;
use tokio::time::sleep;

use crate::{
    a_book_bridge_grpc::{self, ABookBridgeOpenPositionGrpcRequest, ABookBridgePositionSide},
    accounts_manager_grpc::{
        AccountManagerGetClientAccountGrpcRequest, AccountManagerUpdateAccountBalanceGrpcRequest,
        AccountsManagerOperationResult, UpdateBalanceReason,
    },
    position_manager_grpc::PositionManagerOpenPositionGrpcRequest,
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorOpenPositionGrpcRequest,
    },
    validate_instrument_day_off, validate_timeout, AppContext, TradingExecutorError,
};
use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity, TradingProfileNoSqlEntity,
};
use service_sdk::my_telemetry;

pub async fn open_position(
    app: &Arc<AppContext>,
    request: TradingExecutorOpenPositionGrpcRequest,
    telemetry_context: &my_telemetry::MyTelemetryContext,
) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {
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

    validate_instrument_day_off(&target_instrument)?;

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

    validate_timeout(
        app,
        &target_instrument,
        &target_instrument.base,
        &target_instrument.quote,
        &target_account.currency,
    )
    .await?;

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

    let delay = delay_open(
        target_trading_profile_instrument.open_position_min_delay_ms,
        target_trading_profile_instrument.open_position_max_delay_ms,
    )
    .await;

    
    println!("Open delay: {} ms", delay);
    sleep(Duration::from_millis(delay as u64)).await;

    if target_trading_profile.is_a_book {
        let Some(a_book_bridge_grpc_client) = &app.a_book_bridge_grpc_client else {
            return Err(TradingExecutorError::ABookReject);
        };

        let side: ABookBridgePositionSide = request.side().into();
        let a_book_request = ABookBridgeOpenPositionGrpcRequest {
            instrument_id: request.asset_pair.to_string(),
            position_id: position_id.to_string(),
            account_id: request.account_id.to_string(),
            leverage: request.leverage as f64,
            invest_amount: request.invest_amount,
            side: side as i32,
        };

        let response = a_book_bridge_grpc_client
            .open_position(a_book_request.clone(), telemetry_context)
            .await
            .unwrap();

        trade_log::trade_log!(
            &request.trader_id,
            &request.account_id,
            &request.process_id,
            "n/a",
            "Calling ABookBridge open position",
            telemetry_context.clone(),
            "request" = &a_book_request,
            "response" = &response
        );

        let result = {
            if response.status_code == 0 {
                Ok(response.position.unwrap())
            } else {
                Err(TradingExecutorError::ABookReject)
            }
        };
    }

    let balance_update_request = AccountManagerUpdateAccountBalanceGrpcRequest {
        trader_id: request.trader_id.clone(),
        account_id: request.account_id.clone(),
        delta: -request.invest_amount,
        comment: "Open position balance charge".to_string(),
        process_id: request.process_id.clone(),
        allow_negative_balance: false,
        reason: UpdateBalanceReason::TradingResult as i32,
        reference_transaction_id: None,
    };

    let balance_update_result = app
        .accounts_manager_grpc_client
        .update_client_account_balance(
            balance_update_request.clone(),
            &my_telemetry::MyTelemetryContext::new(),
        )
        .await
        .unwrap();

        trade_log::trade_log!(
            &request.trader_id,
            &request.account_id,
            &request.process_id,
            "n/a",
            "Called account manager for updating balance",
            telemetry_context.clone(),
            "request" = &balance_update_request,
            "response" = &balance_update_result
        );

    if AccountsManagerOperationResult::Ok != balance_update_result.result() {
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

    let response = match app
        .position_manager_grpc_client
        .open_position(open_position_request.clone(), telemetry_context)
        .await
    {
        Ok(response) => {

            trade_log::trade_log!(
                &request.trader_id,
                &request.account_id,
                &request.process_id,
                "n/a",
                "Success open position request.",
                telemetry_context.clone(),
                "request" = &open_position_request,
                "response" = &response
            );

            let position: TradingExecutorActivePositionGrpcModel =
                response.position.unwrap().into();

            Ok(position)
        }
        Err(err) => {
            let return_request = AccountManagerUpdateAccountBalanceGrpcRequest {
                trader_id: request.trader_id.clone(),
                account_id: request.account_id.clone(),
                delta: request.invest_amount,
                comment: "Cancel open position balance charge".to_string(),
                process_id: request.process_id.clone(),
                allow_negative_balance: false,
                reason: UpdateBalanceReason::TradingResult as i32,
                reference_transaction_id: None,
            };

            app.accounts_manager_grpc_client
                .update_client_account_balance(
                    return_request.clone(),
                    &my_telemetry::MyTelemetryContext::new(),
                )
                .await
                .unwrap();

                trade_log::trade_log!(
                    &request.trader_id,
                    &request.account_id,
                    &request.process_id,
                    "n/a",
                    "Failed to open position. Returning charged funds.",
                    telemetry_context.clone(),
                    "request" = &open_position_request,
                    "err" = &format!("{:?}", err),
                    "balance_return_request" = &return_request
                );

            return Err(TradingExecutorError::TechError);
        }
    };

    return response;
}

async fn delay_open(from: i32, to: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let delay = rng.gen_range(from..to);

    return delay;
}
