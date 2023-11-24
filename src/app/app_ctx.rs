use std::sync::Arc;

use service_sdk::{
    my_grpc_extensions::GrpcClientSettings, my_no_sql_sdk::reader::MyNoSqlDataReader,
    ServiceContext,
};

use crate::{
    ABookBridgeGrpcClient, AccountsManagerGrpcClient, PositionManagerGrpcClient, SettingsReader,
};
use my_nosql_contracts::{
    TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity, TradingProfileNoSqlEntity,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct AppContext {
    pub position_manager_grpc_client: Arc<PositionManagerGrpcClient>,
    pub accounts_manager_grpc_client: Arc<AccountsManagerGrpcClient>,
    pub a_book_bridge_grpc_client: Arc<ABookBridgeGrpcClient>,
    pub trading_instruments_reader:
        Arc<dyn MyNoSqlDataReader<TradingInstrumentNoSqlEntity> + Send + Sync + 'static>,
    pub trading_groups_reader:
        Arc<dyn MyNoSqlDataReader<TradingGroupNoSqlEntity> + Send + Sync + 'static>,
    pub trading_profiles_reader:
        Arc<dyn MyNoSqlDataReader<TradingProfileNoSqlEntity> + Send + Sync + 'static>,
}

impl AppContext {
    pub async fn new(
        settings: Arc<SettingsReader>,
        service_context: &ServiceContext,
    ) -> AppContext {
        let settings = settings.get_settings().await;
        let position_manager_grpc_client = Arc::new(PositionManagerGrpcClient::new(
            GrpcSettings::new_arc(settings.position_manager_grpc.to_string()),
        ));

        let accounts_manager_grpc_client = Arc::new(AccountsManagerGrpcClient::new(
            GrpcSettings::new_arc(settings.accounts_manager_grpc.to_string()),
        ));

        let a_book_bridge_grpc_client = Arc::new(ABookBridgeGrpcClient::new(
            GrpcSettings::new_arc(settings.a_book_bridge_grpc.to_string()),
        ));

        let trading_instruments_reader = service_context.get_ns_reader().await;
        let trading_groups_reader = service_context.get_ns_reader().await;
        let trading_profiles_reader = service_context.get_ns_reader().await;

        AppContext {
            position_manager_grpc_client,
            accounts_manager_grpc_client,
            trading_instruments_reader,
            trading_groups_reader,
            trading_profiles_reader,
            a_book_bridge_grpc_client,
        }
    }
}

pub struct GrpcSettings(String);

impl GrpcSettings {
    pub fn new_arc(url: String) -> Arc<Self> {
        Arc::new(Self(url))
    }
}

#[tonic::async_trait]
impl GrpcClientSettings for GrpcSettings {
    async fn get_grpc_url(&self, _: &'static str) -> String {
        self.0.clone()
    }
}
