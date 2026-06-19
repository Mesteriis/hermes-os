use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersonTrustError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Obligation(#[from] crate::domains::obligations::ObligationStoreError),

    #[error(transparent)]
    RiskEngine(#[from] crate::engines::risk::RiskEngineError),

    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),
}
