use crate::{
    position_manager_grpc::{
        PositionManagerActivePositionGrpcModel, PositionManagerBidAsk,
        PositionManagerClosedPositionGrpcModel,
    },
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorBidAsk,
        TradingExecutorClosedPositionGrpcModel,
    },
    TradingExecutorError,
};

impl Into<TradingExecutorActivePositionGrpcModel> for PositionManagerActivePositionGrpcModel {
    fn into(self) -> TradingExecutorActivePositionGrpcModel {
        TradingExecutorActivePositionGrpcModel {
            id: self.id,
            account_id: self.account_id,
            trader_id: self.trader_id,
            asset_pair: self.asset_pair,
            side: self.side,
            invest_amount: self.invest_amount,
            leverage: self.leverage,
            stop_out_percent: self.stop_out_percent,
            create_process_id: self.create_process_id,
            create_date_unix_timestamp_milis: self.create_date_unix_timestamp_milis,
            last_update_process_id: self.last_update_process_id,
            last_update_date: self.last_update_date,
            tp_in_profit: self.tp_in_profit,
            sl_in_profit: self.sl_in_profit,
            tp_in_asset_price: self.tp_in_asset_price,
            sl_in_asset_price: self.sl_in_asset_price,
            open_price: self.open_price,
            open_bid_ask: Some(self.open_bid_ask.unwrap().into()),
            open_process_id: self.open_process_id,
            open_date: self.open_date,
            profit: self.profit,
        }
    }
}

impl Into<TradingExecutorClosedPositionGrpcModel> for PositionManagerClosedPositionGrpcModel {
    fn into(self) -> TradingExecutorClosedPositionGrpcModel {
        TradingExecutorClosedPositionGrpcModel {
            id: self.id,
            asset_pair: self.asset_pair,
            side: self.side,
            invest_amount: self.invest_amount,
            leverage: self.leverage,
            stop_out_percent: self.stop_out_percent,
            create_process_id: self.create_process_id,
            create_date_unix_timestamp_milis: self.create_date_unix_timestamp_milis,
            last_update_process_id: self.last_update_process_id,
            last_update_date: self.last_update_date,
            tp_in_profit: self.tp_in_profit,
            sl_in_profit: self.sl_in_profit,
            tp_in_asset_price: self.tp_in_asset_price,
            sl_in_asset_price: self.sl_in_asset_price,
            open_price: self.open_price,
            open_bid_ask: Some(self.open_bid_ask.unwrap().into()),
            open_process_id: self.open_process_id,
            open_date: self.open_date,
            profit: self.profit,
            close_price: self.close_price,
            close_bid_ask: Some(self.close_bid_ask.unwrap().into()),
            close_process_id: self.close_process_id,
            close_reason: self.close_reason,
        }
    }
}

impl Into<TradingExecutorBidAsk> for PositionManagerBidAsk {
    fn into(self) -> TradingExecutorBidAsk {
        TradingExecutorBidAsk {
            asset_pair: self.asset_pair,
            bid: self.bid,
            ask: self.ask,
            date_time_unix_timestamp_milis: self.date_time_unix_timestamp_milis,
        }
    }
}

impl From<i32> for TradingExecutorError {
    fn from(value: i32) -> Self {
        match value {
            0 => panic!("Can't convert 0 to TradingExecutorError"),
            1 => TradingExecutorError::NoLiquidity,
            2 => TradingExecutorError::PositionNotFound,
            _ => panic!("Invalud operation code from position manager"),
        }
    }
}
