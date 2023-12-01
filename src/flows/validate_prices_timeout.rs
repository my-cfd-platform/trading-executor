use chrono::Utc;
use my_nosql_contracts::{BidAskSnapshotNoSqlEntity, TradingInstrumentNoSqlEntity};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{AppContext, TradingExecutorError};

pub async fn validate_timeout(
    app: &AppContext,
    asset_instrument: &TradingInstrumentNoSqlEntity,
    base: &str,
    quote: &str,
    collateral: &str,
) -> Result<(), TradingExecutorError> {
    let Some(bid_asks) = app
        .bid_ask_snapshot_ns_reader
        .get_table_snapshot_as_vec()
        .await
    else {
        return Err(TradingExecutorError::NoLiquidity);
    };

    //check base-collateral
    if base != collateral {
        let Some(base_collateral) = bid_asks.iter().find(|x| {
            (x.base == base && x.quote == collateral) || x.base == collateral && x.quote == base
        }) else {
            return Err(TradingExecutorError::NoLiquidity);
        };

        let Some(instrument) = app
            .trading_instruments_reader
            .get_entity(
                TradingInstrumentNoSqlEntity::generate_partition_key(),
                &base_collateral.row_key,
            )
            .await
        else {
            return Err(TradingExecutorError::InstrumentNotFound);
        };

        validate_instrument_timeout(&instrument, base_collateral.unix_timestamp_with_milis)?;
    }

    if quote != collateral {
        let Some(quote_collateral) = bid_asks.iter().find(|x| {
            (x.base == quote && x.quote == collateral) || x.base == collateral && x.quote == quote
        }) else {
            return Err(TradingExecutorError::NoLiquidity);
        };

        let Some(instrument) = app
            .trading_instruments_reader
            .get_entity(
                TradingInstrumentNoSqlEntity::generate_partition_key(),
                &quote_collateral.row_key,
            )
            .await
        else {
            return Err(TradingExecutorError::InstrumentNotFound);
        };

        validate_instrument_timeout(&instrument, quote_collateral.unix_timestamp_with_milis)?;
    }

    let Some(asset_bidask) = bid_asks.iter().find(|x| x.row_key == asset_instrument.get_id()) else {
        return Err(TradingExecutorError::NoLiquidity);
    };



    let Some(instrument) = app
        .trading_instruments_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &asset_bidask.row_key,
        )
        .await
    else {
        return Err(TradingExecutorError::InstrumentNotFound);
    };

    validate_instrument_timeout(instrument.as_ref(), asset_bidask.unix_timestamp_with_milis)?;
    return Ok(());
}


pub fn validate_instrument_timeout(
    instrument: &TradingInstrumentNoSqlEntity,
    last_bidask_date: u64
) -> Result<(), TradingExecutorError> {
    let current_date = Utc::now();
    let ns_date: DateTimeAsMicroseconds = last_bidask_date.into();
    let ns_date = ns_date.to_chrono_utc();

    if let Some(day_t) = instrument.day_timeout {
        let timeout = chrono::Duration::seconds(day_t as i64);
        let diff = current_date - ns_date;
        if diff > timeout {
            return Err(TradingExecutorError::NoLiquidity);
        }

        return Ok(());
    }

    if let Some(day_n) = instrument.night_timeout {
        let timeout = chrono::Duration::seconds(day_n as i64);
        let diff = current_date - ns_date;
        if diff > timeout {
            return Err(TradingExecutorError::NoLiquidity);
        }

        return Ok(());
    }

    return Ok(());
}