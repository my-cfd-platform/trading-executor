use crate::{TradingExecutorError, trading_executor_grpc::TradingExecutorOperationsCodes};

impl Into<TradingExecutorOperationsCodes> for TradingExecutorError {
    fn into(self) -> TradingExecutorOperationsCodes {
        match self{
            TradingExecutorError::DayOff => TradingExecutorOperationsCodes::DayOff,
            TradingExecutorError::OperationIsTooLow => TradingExecutorOperationsCodes::OperationIsTooLow,
            TradingExecutorError::OperationIsTooHigh => TradingExecutorOperationsCodes::OperationIsTooHigh,
            TradingExecutorError::MinOperationsByInstrumentViolated => TradingExecutorOperationsCodes::MinOperationsByInstrumentViolated,
            TradingExecutorError::MaxOperationsByInstrumentViolated => TradingExecutorOperationsCodes::MaxOperationsByInstrumentViolated,
            TradingExecutorError::NotEnoughBalance => TradingExecutorOperationsCodes::NotEnoughBalance,
            TradingExecutorError::NoLiquidity => TradingExecutorOperationsCodes::NoLiquidity,
            TradingExecutorError::PositionNotFound => TradingExecutorOperationsCodes::PositionNotFound,
            TradingExecutorError::TpIsTooClose => TradingExecutorOperationsCodes::TpIsTooClose,
            TradingExecutorError::SlIsTooClose => TradingExecutorOperationsCodes::SlIsTooClose,
            TradingExecutorError::AccountNotFound => TradingExecutorOperationsCodes::AccountNotFound,
            TradingExecutorError::InstrumentNotFound => TradingExecutorOperationsCodes::InstrumentNotFound,
            TradingExecutorError::InstrumentIsNotTradable => TradingExecutorOperationsCodes::InstrumentIsNotTradable,
            TradingExecutorError::HitMaxAmountOfPendingOrders => TradingExecutorOperationsCodes::HitMaxAmountOfPendingOrders,
            TradingExecutorError::TechError => TradingExecutorOperationsCodes::TechError,
            TradingExecutorError::MultiplierIsNotFound => TradingExecutorOperationsCodes::MultiplierIsNotFound,
            TradingExecutorError::TradingDisabled => TradingExecutorOperationsCodes::TradingDisabled,
            TradingExecutorError::MaxPositionsAmount => TradingExecutorOperationsCodes::MaxPositionsAmount,
            TradingExecutorError::TradingGroupNotFound => TradingExecutorOperationsCodes::TradingGroupNotFound,
            TradingExecutorError::TradingProfileNotFound => TradingExecutorOperationsCodes::TradingProfileNotFound,
            TradingExecutorError::TradingProfileInstrumentNotFound => TradingExecutorOperationsCodes::TradingProfileInstrumentNotFound,
            TradingExecutorError::ABookReject => TradingExecutorOperationsCodes::ABookReject,
        }
    }
}