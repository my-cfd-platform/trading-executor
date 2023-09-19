use std::sync::Arc;

use trading_executor::{
    trading_executor_grpc::trading_executor_grpc_service_server::TradingExecutorGrpcServiceServer,
    AppContext, GrpcService, SettingsReader, 
};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;
    let app_context = Arc::new(AppContext::new(settings_reader.clone(), &service_context).await);

    service_context.configure_grpc_server(|builder| {
        builder.add_grpc_service(TradingExecutorGrpcServiceServer::new(GrpcService::new(
            app_context.clone(),
        )));
    });

    service_context.start_application().await;
}
