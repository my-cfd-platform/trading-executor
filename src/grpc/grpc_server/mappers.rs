use crate::TradingExecutorError;

pub fn map_error_to_grpc_status(error: &TradingExecutorError) -> i32 {
    match error {
        TradingExecutorError::DayOff => 1,
        TradingExecutorError::OperationIsTooLow => 2,
        TradingExecutorError::OperationIsTooHigh => 3,
        TradingExecutorError::MinOperationsByInstrumentViolated => 4,
        TradingExecutorError::MaxOperationsByInstrumentViolated => 5,
        TradingExecutorError::NotEnoughBalance => 6,
        TradingExecutorError::NoLiquidity => 7,
        TradingExecutorError::PositionNotFound => 8,
        TradingExecutorError::TpIsTooClose => 9,
        TradingExecutorError::SlIsTooClose => 10,
        TradingExecutorError::AccountNotFound => 11,
        TradingExecutorError::InstrumentNotFound => 12,
        TradingExecutorError::InstrumentIsNotTradable => 13,
        TradingExecutorError::HitMaxAmountOfPendingOrders => 14,
        TradingExecutorError::TechError => 15,
        TradingExecutorError::MultiplierIsNotFound => 16,
        TradingExecutorError::TradingDisabled => 17,
        TradingExecutorError::MaxPositionsAmount => 18,
        TradingExecutorError::TradingGroupNotFound => 19,
        TradingExecutorError::TradingProfileNotFound => 20,
        TradingExecutorError::TradingProfileInstrumentNotFound => 21,
    }
}
