use chrono::Utc;
use my_nosql_contracts::{TradingInstrumentNoSqlEntity, BidAskSnapshotNoSqlEntity};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{AppContext, TradingExecutorError};

pub async fn validate_timeout(
    app: &AppContext,
    asset_pair: &str,
    instrument: &TradingInstrumentNoSqlEntity,
) -> Result<(), TradingExecutorError> {
    let target_bidask = app
        .bid_ask_snapshot_ns_reader
        .get_entity(
            BidAskSnapshotNoSqlEntity::generate_partition_key(),
            asset_pair,
        )
        .await;

    let Some(target_bidask) = target_bidask else {
        return Ok(());
    };

    let current_date = Utc::now();
    let ns_date: DateTimeAsMicroseconds = target_bidask.unix_timestamp_with_milis.into();
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
