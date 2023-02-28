use std::{net::SocketAddr, sync::Arc};

use tonic::transport::Server;

use crate::{
    trading_executor_grpc::trading_executor_grpc_service_server::TradingExecutorGrpcServiceServer,
    AppContext,
};

#[derive(Clone)]
pub struct GrpcService {
    pub app: Arc<AppContext>,
}

impl GrpcService {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

pub async fn start_grpc_server(app: Arc<AppContext>, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let service = GrpcService::new(app.clone());

    println!("Listening to {:?} as grpc endpoint", addr);

    Server::builder()
        .add_service(TradingExecutorGrpcServiceServer::new(service.clone()))
        .serve(addr)
        .await
        .unwrap();
}
