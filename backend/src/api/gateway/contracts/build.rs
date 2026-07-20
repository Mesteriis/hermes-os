fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    let mut config = prost_build::Config::new();
    config.extern_path(".hermes.runtime.v1", "::hermes_runtime_protocol::v1");
    config
        .compile_protos(
            &[
                "proto/hermes/gateway/v1/recovery.proto",
                "proto/hermes/gateway/v1/owner_control.proto",
                "proto/hermes/gateway/v1/module_registration.proto",
                "proto/hermes/gateway/v1/external_runtime_session.proto",
                "proto/hermes/gateway/v1/client_realtime.proto",
                "proto/hermes/gateway/v1/browser_session.proto",
                "proto/hermes/gateway/v1/client_bootstrap.proto",
            ],
            &["proto", "../../../platform/runtime_protocol/proto"],
        )
        .expect("gateway protocol must compile");
}
