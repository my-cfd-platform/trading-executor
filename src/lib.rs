mod app;
mod flows;
mod grpc;
mod models;

pub mod position_manager_grpc {
    tonic::include_proto!("position_manager");
}
pub mod trading_executor_grpc {
    tonic::include_proto!("trading_executor");
}
pub mod accounts_manager_grpc {
    tonic::include_proto!("accounts_manager");
}

pub use app::*;
pub use flows::*;
pub use grpc::*;
pub use models::*;
