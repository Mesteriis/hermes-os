use std::process::ExitCode;

use hermes_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_CONTRACT_MAJOR_V1, COMMUNICATION_BULK_ACTION_MAX_TARGETS_V1,
};

fn main() -> ExitCode {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    match (arguments.next(), arguments.next()) {
        (Some(command), None) if command == "validate-contracts" => validate_contracts(),
        _ => {
            eprintln!(
                "bulk action managed adapter is not admitted; available command: validate-contracts"
            );
            ExitCode::from(64)
        }
    }
}

fn validate_contracts() -> ExitCode {
    if COMMUNICATION_BULK_ACTION_CONTRACT_MAJOR_V1 == 1
        && COMMUNICATION_BULK_ACTION_MAX_TARGETS_V1 == 100
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
