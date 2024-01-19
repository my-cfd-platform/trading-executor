use serde::{Deserialize, Serialize};
service_sdk::macros::use_settings!();

#[derive(
    my_settings_reader::SettingsModel,
    AutoGenerateSettingsTraits,
    SdkSettingsTraits,
    Serialize,
    Deserialize,
    Debug,
    Clone,
)]
pub struct SettingsModel {
    pub accounts_manager_grpc: String,
    pub position_manager_grpc: String,
    pub a_book_bridge_grpc: Option<String>,
    pub my_no_sql_tcp_reader: String,
    pub seq_conn_string: String,
    pub my_telemetry: String,
}
