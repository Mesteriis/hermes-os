fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    prost_build::Config::new()
        .compile_protos(&["proto/hermes/zulip/v1/client.proto"], &["proto"])
        .expect("Zulip client protocol must compile");
}
