use serde::{Deserialize, Serialize};
use service_sdk::async_trait;
service_sdk::macros::use_settings!();

#[derive(
    my_settings_reader::SettingsModel, SdkSettingsTraits, Serialize, Deserialize, Debug, Clone,
)]
pub struct SettingsModel {
    pub accounts_manager_grpc: String,
    pub position_manager_grpc: String,
    pub a_book_bridge_grpc: String,
    pub no_sql_tcp: String,
    pub seq_conn_string: String,
    pub my_telemetry: String,
}

#[async_trait::async_trait]
impl MyNoSqlTcpConnectionSettings for SettingsReader {
    async fn get_host_port(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.no_sql_tcp.clone()
    }
}

#[async_trait::async_trait]
impl service_sdk::my_telemetry::my_telemetry_writer::MyTelemetrySettings for SettingsReader {
    async fn get_telemetry_url(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.my_telemetry.clone()
    }
}

#[async_trait::async_trait]
impl service_sdk::my_logger::my_seq_logger::SeqSettings for SettingsReader {
    async fn get_conn_string(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.seq_conn_string.clone()
    }
}
