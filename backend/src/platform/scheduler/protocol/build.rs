fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    prost_build::compile_protos(&["proto/hermes/scheduler/v1/job_command.proto"], &["proto"])
        .expect("scheduler job protocol must compile");
}
