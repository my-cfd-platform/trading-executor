fn main() {
    tonic_build::compile_protos("proto/positions_manager_grpc_service.proto").unwrap();
    tonic_build::compile_protos("proto/trading_executor_grpc_service.proto").unwrap();
    tonic_build::compile_protos("proto/accounts_manager_grcp_service.proto").unwrap();
}