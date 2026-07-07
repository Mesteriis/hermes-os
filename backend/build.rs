fn main() {
    println!("cargo:rerun-if-env-changed=HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON");
    println!("cargo:rerun-if-env-changed=HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=HERMES_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET");
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=../contracts/proto/hermes/common/v1/common.proto");
    println!("cargo:rerun-if-changed=../contracts/proto/hermes/signal_hub/v1/signal_hub.proto");
    println!(
        "cargo:rerun-if-changed=../contracts/proto/hermes/communications/v1/communications.proto"
    );
    println!("cargo:rerun-if-changed=../contracts/proto");
    connectrpc_build::Config::new()
        .files(&[
            "../contracts/proto/hermes/common/v1/common.proto",
            "../contracts/proto/hermes/signal_hub/v1/signal_hub.proto",
            "../contracts/proto/hermes/communications/v1/communications.proto",
        ])
        .includes(&["../contracts/proto"])
        .include_file("_connectrpc.rs")
        .compile()
        .expect("connectrpc codegen should succeed");
}
