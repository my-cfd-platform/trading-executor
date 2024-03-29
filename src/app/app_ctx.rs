use std::sync::Arc;

use service_sdk::{
    my_grpc_extensions::GrpcClientSettings, my_no_sql_sdk::reader::MyNoSqlDataReaderTcp,
    ServiceContext,
};

use crate::{ABookBridgeGrpcClient, AccountsManagerGrpcClient, PositionManagerGrpcClient};
use my_nosql_contracts::{
    BidAskSnapshotNoSqlEntity, TradingGroupNoSqlEntity, TradingInstrumentNoSqlEntity,
    TradingProfileNoSqlEntity,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct AppContext {
    pub position_manager_grpc_client: Arc<PositionManagerGrpcClient>,
    pub accounts_manager_grpc_client: Arc<AccountsManagerGrpcClient>,
    pub a_book_bridge_grpc_client: Option<Arc<ABookBridgeGrpcClient>>,
    pub trading_instruments_reader: Arc<MyNoSqlDataReaderTcp<TradingInstrumentNoSqlEntity>>,
    pub trading_groups_reader: Arc<MyNoSqlDataReaderTcp<TradingGroupNoSqlEntity>>,
    pub trading_profiles_reader: Arc<MyNoSqlDataReaderTcp<TradingProfileNoSqlEntity>>,
    pub bid_ask_snapshot_ns_reader: Arc<MyNoSqlDataReaderTcp<BidAskSnapshotNoSqlEntity>>,
}

impl AppContext {
    pub async fn new(
        settings: Arc<crate::settings::SettingsReader>,
        service_context: &ServiceContext,
    ) -> AppContext {
        let settings = settings.get_settings().await;
        let position_manager_grpc_client = Arc::new(PositionManagerGrpcClient::new(
            GrpcSettings::new_arc(settings.position_manager_grpc.to_string()),
        ));

        let accounts_manager_grpc_client = Arc::new(AccountsManagerGrpcClient::new(
            GrpcSettings::new_arc(settings.accounts_manager_grpc.to_string()),
        ));

        let a_book_bridge_grpc_client = settings.a_book_bridge_grpc.map(|x| {
            Arc::new(ABookBridgeGrpcClient::new(GrpcSettings::new_arc(
                x.to_string(),
            )))
        });

        let trading_instruments_reader = service_context.get_ns_reader().await;
        let trading_groups_reader = service_context.get_ns_reader().await;
        let trading_profiles_reader = service_context.get_ns_reader().await;
        let bid_ask_snapshot_ns_reader = service_context.get_ns_reader().await;

        AppContext {
            position_manager_grpc_client,
            accounts_manager_grpc_client,
            trading_instruments_reader,
            trading_groups_reader,
            trading_profiles_reader,
            a_book_bridge_grpc_client,
            bid_ask_snapshot_ns_reader,
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
