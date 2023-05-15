use std::sync::Arc;

use my_no_sql_tcp_reader::{MyNoSqlDataReader, MyNoSqlTcpConnection};
use rust_extensions::AppStates;

use crate::{AccountsManagerGrpcClient, PositionManagerGrpcClient, SettingsModel, ABookBridgeGrpcClient};
use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity, TradingProfileNoSqlEntity,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub position_manager_grpc_client: Arc<PositionManagerGrpcClient>,
    pub accounts_manager_grpc_client: Arc<AccountsManagerGrpcClient>,
    pub a_book_bridge_grpc_client: Arc<ABookBridgeGrpcClient>,
    pub trading_instruments_reader: Arc<MyNoSqlDataReader<TradingInstrumentNoSqlEntity>>,
    pub trading_groups_reader: Arc<MyNoSqlDataReader<TradingGroupNoSqlEntity>>,
    pub trading_profiles_reader: Arc<MyNoSqlDataReader<TradingProfileNoSqlEntity>>,
    pub my_no_sql_connection: MyNoSqlTcpConnection,
}

impl AppContext {
    pub async fn new(settings: Arc<SettingsModel>) -> AppContext {
        let position_manager_grpc_client = Arc::new(
            PositionManagerGrpcClient::new(settings.position_manager_grpc.to_string()).await,
        );

        let accounts_manager_grpc_client = Arc::new(
            AccountsManagerGrpcClient::new(settings.accounts_manager_grpc.to_string()).await,
        );

        let a_book_bridge_grpc_client = Arc::new(
            ABookBridgeGrpcClient::new(settings.a_book_bridge_grpc.to_string()).await,
        );

        let my_no_sql_connection = my_no_sql_tcp_reader::MyNoSqlTcpConnection::new(
            format!("{}:{}", crate::app::APP_NAME, crate::app::APP_VERSION),
            settings.clone(),
        );

        let trading_instruments_reader = my_no_sql_connection.get_reader().await;
        let trading_groups_reader = my_no_sql_connection.get_reader().await;
        let trading_profiles_reader = my_no_sql_connection.get_reader().await;

        AppContext {
            position_manager_grpc_client,
            accounts_manager_grpc_client,
            trading_instruments_reader,
            trading_groups_reader,
            trading_profiles_reader,
            my_no_sql_connection,
            app_states: Arc::new(AppStates::create_initialized()),
            a_book_bridge_grpc_client
        }
    }
}
