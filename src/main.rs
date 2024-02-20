mod app;
mod flows;
mod grpc;
mod models;
mod settings;
use service_sdk::ServiceInfo;
pub mod position_manager_grpc {
    tonic::include_proto!("position_manager");
}
pub mod trading_executor_grpc {
    tonic::include_proto!("trading_executor");
}
pub mod accounts_manager_grpc {
    tonic::include_proto!("accounts_manager");
}
pub mod a_book_bridge_grpc {
    tonic::include_proto!("a_book_bridge");
}

pub use app::*;
pub use flows::*;
pub use grpc::*;
pub use models::*;

use std::sync::Arc;

use trading_executor_grpc::trading_executor_grpc_service_server::TradingExecutorGrpcServiceServer;

#[tokio::main]
async fn main() {
    let settings_reader = crate::settings::SettingsReader::new(".my-cfd").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;
    let app_context = Arc::new(AppContext::new(settings_reader.clone(), &service_context).await);

    service_context.configure_grpc_server(|builder| {
        builder.add_grpc_service(TradingExecutorGrpcServiceServer::new(GrpcService::new(
            app_context.clone(),
        )));
    });

    trade_log::core::TRADE_LOG
        .init_component_name(settings_reader.get_service_name().as_str())
        .await;
    trade_log::core::TRADE_LOG
        .start(&service_context.sb_client)
        .await;

    service_context.start_application().await;
}
