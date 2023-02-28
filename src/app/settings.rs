use my_no_sql_tcp_reader::MyNoSqlTcpConnectionSettings;
use my_settings_reader::SettingsModel;
use serde::{Serialize, Deserialize};

#[derive(SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "AccountsManagerGrpc")]
    pub accounts_manager_grpc: String,
    #[serde(rename = "PositionManagerGrpc")]
    pub position_manager_grpc: String,
    #[serde(rename = "NoSqlTcp")]
    pub no_sql_tcp: String,
}

#[async_trait::async_trait]
impl MyNoSqlTcpConnectionSettings for SettingsModel {
    async fn get_host_port(&self) -> String{
        self.no_sql_tcp.clone()
    }
}