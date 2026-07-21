//! Owner-authorized offline whole-instance capture and restore CLI dispatch.

mod capture;
mod keys;
mod restore;
mod staging;

use std::path::PathBuf;

use crate::cli::WholeInstanceRecoveryCommand;

pub(crate) fn run(
    data_dir: Option<PathBuf>,
    operation: WholeInstanceRecoveryCommand,
) -> Result<(), String> {
    match operation {
        WholeInstanceRecoveryCommand::Capture(capture) => {
            capture::capture_instance(data_dir, capture)
        }
        WholeInstanceRecoveryCommand::Restore(restore) => {
            restore::restore_instance(data_dir, restore)
        }
    }
}
