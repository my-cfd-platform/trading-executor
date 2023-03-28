use std::sync::Arc;

use my_nosql_contracts::TradingInstrumentNoSqlEntity;

use crate::{
    trading_executor_grpc::{
        TradingExecutorClosePositionGrpcRequest, TradingExecutorClosedPositionGrpcModel,
    },
    AppContext, TradingExecutorError,
};

pub async fn close_position(
    app: &Arc<AppContext>,
    request: TradingExecutorClosePositionGrpcRequest,
) -> Result<TradingExecutorClosedPositionGrpcModel, TradingExecutorError> {
    let Some(_) = app
        .accounts_manager_grpc_client
        .get_client_account(&request.trader_id, &request.account_id)
        .await.account else{
            return Err(TradingExecutorError::AccountNotFound)
        };

    let Some(target_position) = app.position_manager_grpc_client.get_active_position(&request.trader_id, &request.account_id, &request.position_id).await else{
        return Err(TradingExecutorError::PositionNotFound);
    };

    let target_instrument = app
        .trading_instruments_reader
        .get_entity(
            TradingInstrumentNoSqlEntity::generate_partition_key(),
            &target_position.asset_pair,
        )
        .await;

    let Some(_) = target_instrument else{
        return Err(TradingExecutorError::InstrumentNotFound)
    };

    return app
        .position_manager_grpc_client
        .close_position(
            &request.trader_id,
            &request.account_id,
            &request.position_id,
            &request.process_id,
        )
        .await;
}
