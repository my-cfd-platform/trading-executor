fn main() {

    let url = "https://raw.githubusercontent.com/my-cfd-platform/proto-files/main/proto/";
    ci_utils::sync_and_build_proto_file(url, "TradingExecutorGrpcService.proto");
    ci_utils::sync_and_build_proto_file(url, "ABookBridge.proto");
    ci_utils::sync_and_build_proto_file(url, "PositionsManager.proto");

    tonic_build::compile_protos("proto/accounts_manager_grcp_service.proto").unwrap();
}