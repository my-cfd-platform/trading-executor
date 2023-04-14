use std::sync::Arc;

use my_nosql_contracts::{TradingGroupNoSqlEntity, TradingProfileNoSqlEntity};

use crate::{ 
    position_manager_grpc::PositionManagerUpdateSlTpGrpcRequest,
    trading_executor_grpc::{
        TradingExecutorActivePositionGrpcModel, TradingExecutorUpdateSlTpGrpcRequest,
    },
    AppContext, TradingExecutorError,
};

pub async fn update_sl_tp(
    app: &Arc<AppContext>,
    request: TradingExecutorUpdateSlTpGrpcRequest,
) -> Result<TradingExecutorActivePositionGrpcModel, TradingExecutorError> {

    let Some(target_account) = app
        .accounts_manager_grpc_client
        .get_client_account(&request.trader_id, &request.account_id)
        .await.account else{
            return Err(TradingExecutorError::AccountNotFound)
        };

    let Some(target_trading_group) = app.trading_groups_reader.get_entity(TradingGroupNoSqlEntity::generate_partition_key(), &target_account.trading_group).await else{
        return Err(TradingExecutorError::TradingGroupNotFound)
    };

    let Some(_) = app.trading_profiles_reader.get_entity(TradingProfileNoSqlEntity::generate_partition_key(), &target_trading_group.trading_profile_id).await else{
        return Err(TradingExecutorError::TradingProfileNotFound)
    };

    let pm_request = PositionManagerUpdateSlTpGrpcRequest {
        position_id: request.position_id,
        account_id: request.account_id,
        trader_id: request.trader_id,
        tp_in_profit: request.tp_in_profit,
        tp_in_asset_price: request.tp_in_asset_price,
        sl_in_asset_price: request.sl_in_asset_price,
        sl_in_profit: request.sl_in_profit,
        process_id: request.process_id,
    };

    return app
        .position_manager_grpc_client
        .update_sl_tp(pm_request)
        .await;
}
