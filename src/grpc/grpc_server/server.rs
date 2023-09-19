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