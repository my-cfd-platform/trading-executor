use service_sdk::{async_trait, my_grpc_extensions, my_telemetry};

#[service_sdk::my_grpc_extensions::client::generate_grpc_client(
    proto_file: "./proto/ABookBridge.proto",
    crate_ns: "crate::a_book_bridge_grpc",
    retries: 3,
    request_timeout_sec: 10,
    ping_timeout_sec: 1,
    ping_interval_sec: 3,
)]
pub struct ABookBridgeGrpcClient {
}
