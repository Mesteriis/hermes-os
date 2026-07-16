fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    prost_build::compile_protos(&["proto/hermes/runtime/v1/recovery.proto"], &["proto"])
        .expect("runtime recovery protocol must compile");
}
