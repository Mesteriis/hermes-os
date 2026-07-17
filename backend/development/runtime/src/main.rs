mod cli;
mod identity;
mod kernel_operator;
mod modules;
mod pairing;
mod platform;
mod runtime;

mod control_store {
    #[path = "../../../../src/kernel/src/control_store/lifecycle.rs"]
    pub(crate) mod lifecycle;
}

mod distribution {
    #[path = "../../../../src/kernel/src/distribution/staged_artifact.rs"]
    pub(crate) mod staged_artifact;
}

mod infrastructure {
    #[path = "../../../../src/kernel/src/infrastructure/filesystem.rs"]
    pub(crate) mod filesystem;
}

mod recovery {
    #[path = "../../../../src/kernel/src/recovery/fence.rs"]
    pub(crate) mod fence;
}

use clap::Parser;

use cli::operator::Cli;

fn main() {
    if let Err(error) = cli::operator::run(Cli::parse()) {
        eprintln!("development kernel operator failed: {error}");
        std::process::exit(1);
    }
}
