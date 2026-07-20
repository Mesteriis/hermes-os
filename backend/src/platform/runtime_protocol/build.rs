fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    prost_build::compile_protos(
        &[
            "proto/hermes/runtime/v1/recovery.proto",
            "proto/hermes/runtime/v1/deployment.proto",
            "proto/hermes/runtime/v1/distribution.proto",
            "proto/hermes/runtime/v1/telemetry_runtime.proto",
            "proto/hermes/runtime/v1/blob_runtime.proto",
            "proto/hermes/runtime/v1/events_authority_runtime.proto",
            "proto/hermes/runtime/v1/scheduler_runtime.proto",
            "proto/hermes/runtime/v1/vault_runtime.proto",
            "proto/hermes/runtime/v1/managed_runtime_control.proto",
        ],
        &["proto"],
    )
    .expect("runtime recovery protocol must compile");
}
