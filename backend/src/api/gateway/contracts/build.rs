use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    let mut config = prost_build::Config::new();
    config.extern_path(".hermes.runtime.v1", "::hermes_runtime_protocol::v1");
    config.enum_attribute(
        ".hermes.gateway.v1.ExternalRuntimeSessionRequestV1.operation",
        "#[allow(clippy::large_enum_variant)]",
    );
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

    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-query-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["../../../communications-api/proto/hermes/communications/query/v1/query.proto"],
            &["../../../communications-api/proto"],
        )
        .expect("communications query schema must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("communications query descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_query_schema.rs"),
        format!("pub const COMMUNICATIONS_QUERY_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("communications query schema digest must be written");
}
