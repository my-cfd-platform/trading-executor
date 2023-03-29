use std::sync::Arc;

use trading_executor::{start_grpc_server, AppContext, SettingsReader, APP_NAME, APP_VERSION};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd").await;
    let settings_reader = Arc::new(settings_reader.get_settings().await);

    let app = AppContext::new(settings_reader.clone()).await;

    let app = Arc::new(app);

    app.my_no_sql_connection
        .start(my_logger::LOGGER.clone())
        .await;

    http_is_alive_shared::start_up::start_server(
        APP_NAME.to_string(),
        APP_VERSION.to_string(),
        app.app_states.clone(),
    );

    tokio::spawn(start_grpc_server(app.clone(), 8888));

    app.app_states.wait_until_shutdown().await;
}
