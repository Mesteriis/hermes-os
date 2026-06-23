fn main() {
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
