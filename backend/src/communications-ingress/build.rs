use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-observation-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/hermes/communications/ingress/v1/observation.proto"],
            &["proto"],
        )
        .expect("communications ingress protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("communications ingress descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_observation_schema.rs"),
        format!("pub const COMMUNICATION_OBSERVATION_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("communications ingress schema digest must be written");

    compile_schema(
        &output,
        "communications-attachment-blob-admission-observation-v1.bin",
        "communications_attachment_blob_admission_observation_schema.rs",
        "COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256",
        "proto/hermes/communications/ingress/attachment/blob/v1/observation.proto",
    );
    compile_schema(
        &output,
        "communications-attachment-safety-verdict-observation-v1.bin",
        "communications_attachment_safety_verdict_observation_schema.rs",
        "COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256",
        "proto/hermes/communications/ingress/attachment/safety/v1/observation.proto",
    );
    compile_schema(
        &output,
        "communications-attachment-anchor-recorded-v1.bin",
        "communications_attachment_anchor_recorded_schema.rs",
        "COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256",
        "proto/hermes/communications/ingress/attachment/anchor/v1/recorded.proto",
    );
}

fn compile_schema(
    output: &std::path::Path,
    descriptor_name: &str,
    schema_name: &str,
    constant: &str,
    proto: &str,
) {
    let descriptor = output.join(descriptor_name);
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(&[proto], &["proto"])
        .expect("communications attachment observation protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("attachment descriptor must exist"))
            .into();
    std::fs::write(
        output.join(schema_name),
        format!("pub const {constant}: [u8; 32] = {digest:?};\n"),
    )
    .expect("communications attachment observation schema digest must be written");
}
