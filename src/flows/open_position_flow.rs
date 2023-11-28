use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc, Weekday};
use rand::Rng;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    a_book_bridge_grpc::{ABookBridgeOpenPositionGrpcRequest, ABookBridgePositionSide},
    accounts_manager_grpc::{
        AccountManagerGetClientAccountGrpcRequest, AccountManagerUpdateAccountBalanceGrpcRequest,
        AccountsManagerOperationResult, UpdateBalanceReason,
    },
    position_manager_grpc::PositionManagerOpenPositionGrpcRequest,
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorOpenPositionGrpcRequest,
    },
    AppContext, TradingExecutorError,
};
use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentDayOff, TradingInstrumentNoSqlEntity,
    TradingProfileNoSqlEntity,
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

    //open delay
    println!("Open delay: {} ms", delay);
    sleep(Duration::from_millis(delay as u64)).await;

    if target_trading_profile.is_a_book {
        let side: ABookBridgePositionSide = request.side().into();
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
            .open_position(request, telemetry_context)
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

    if AccountsManagerOperationResult::Ok != balance_update_result.unwrap().result() {
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
        .open_position(open_position_request, telemetry_context)
        .await
        .unwrap();

    let position = position.position.unwrap().into();

    return Ok(position);
}

pub fn validate_instrument_day_off(
    instrument: &TradingInstrumentNoSqlEntity,
) -> Result<(), TradingExecutorError> {
    for day_off in &instrument.days_off {
        validate_day_off(&day_off, Utc::now())?;
    }

    return Ok(());
}

pub fn validate_day_off(
    instrument: &TradingInstrumentDayOff,
    current_date: DateTime<Utc>,
) -> Result<(), TradingExecutorError> {
    let current_weekday = current_date.weekday();
    let current_time = current_date.time();

    let from_as_int = as_int(
        convert_csharp_int_day_to_rust_weekday(instrument.dow_from),
        instrument.time_from.parse().unwrap(),
    );

    let to_as_int = as_int(
        convert_csharp_int_day_to_rust_weekday(instrument.dow_to),
        instrument.time_to.parse().unwrap(),
    );

    let current_as_int = as_int(current_weekday, current_time);

    let is_first_case = from_as_int < to_as_int;

    let is_day_off = match is_first_case {
        true => from_as_int <= current_as_int && current_as_int <= to_as_int,
        false => {
            let first_case = current_as_int >= from_as_int;
            let second_case = current_as_int <= to_as_int;
            first_case || second_case
        }
    };

    if is_day_off {
        return Err(TradingExecutorError::DayOff);
    }

    return Ok(());
}

fn as_int(weekday: Weekday, time: NaiveTime) -> u32 {
    return weekday as u32 * 86400 + time.hour() * 3600 + time.minute() * 60 + time.second();
}

fn convert_csharp_int_day_to_rust_weekday(src: i32) -> Weekday {
    if src == 0 {
        return Weekday::Sun;
    }
    let src = src - 1;

    return Weekday::try_from(src as u8).unwrap();
}

async fn delay_open(from: i32, to: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let delay = rng.gen_range(from..to);

    return delay;
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};
    use my_nosql_contracts::TradingInstrumentDayOff;

    use crate::validate_day_off;

    #[test]
    fn check_day_off_f_s_t_day_off() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 1,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 20).and_hms(22, 0, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(true, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_s_t_day_on() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 1,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 20).and_hms(20, 0, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(false, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_b_t_day_off() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 5,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 20).and_hms(22, 0, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(true, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_b_t_day_off_2() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 5,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 22).and_hms(14, 29, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(false, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_b_t_day_on() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 5,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 22).and_hms(14, 31, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(false, validate_result.is_err());
    }
}
