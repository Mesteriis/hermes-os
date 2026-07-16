fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    let mut config = prost_build::Config::new();
    config.extern_path(".hermes.runtime.v1", "::hermes_runtime_protocol::v1");
    config
        .compile_protos(
            &["proto/hermes/gateway/v1/recovery.proto"],
            &["proto", "../../../platform/runtime_protocol/proto"],
        )
        .expect("gateway recovery protocol must compile");
}
