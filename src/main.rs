use std::sync::Arc;

use trading_executor::{start_grpc_server, AppContext, SettingsReader};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd").await;
    let settings_reader = Arc::new(settings_reader.get_settings().await);

    let app = AppContext::new(settings_reader.clone()).await;

    let app = Arc::new(app);

    app.my_no_sql_connection.start().await;

    tokio::spawn(start_grpc_server(app.clone(), 8888));

    app.app_states.wait_until_shutdown().await;
}
