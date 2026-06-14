use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObligationEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("beneficiary entity kind and id must be provided together")]
    PartialBeneficiary,
}
