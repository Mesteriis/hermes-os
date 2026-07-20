//! Gateway-local sessions fenced by the Control Store device identity epoch.

mod session {
    mod browser;
    mod http;
    mod pairing;
    mod service;
    mod webauthn;

    pub use browser::{
        BrowserAuthenticationManager, BrowserSession, BrowserSessionManager,
        BrowserWebauthnAuthenticationCeremonyV1,
    };
    pub use http::BrowserSameOriginSessionV1;
    pub use pairing::{
        BrowserPairingChallengeV1, BrowserPairingManager, BrowserWebauthnPairingCeremonyV1,
        OwnerPairingApprovalV1,
    };
    pub use service::{BrowserGatewayAccessModeV1, BrowserGatewaySessionService};
    pub use webauthn::{
        BrowserAssertionMaterialV1, BrowserAuthenticationCeremonyV1, BrowserCredentialMaterialV1,
        BrowserRegistrationCeremonyV1, BrowserWebauthnVerifier, VerifiedBrowserCredentialV1,
    };
}

pub use session::BrowserSameOriginSessionV1;
pub use session::{
    BrowserAssertionMaterialV1, BrowserAuthenticationCeremonyV1, BrowserCredentialMaterialV1,
    BrowserRegistrationCeremonyV1, BrowserWebauthnVerifier, VerifiedBrowserCredentialV1,
};
pub use session::{
    BrowserAuthenticationManager, BrowserGatewayAccessModeV1, BrowserGatewaySessionService,
    BrowserSession, BrowserSessionManager, BrowserWebauthnAuthenticationCeremonyV1,
};
pub use session::{
    BrowserPairingChallengeV1, BrowserPairingManager, BrowserWebauthnPairingCeremonyV1,
    OwnerPairingApprovalV1,
};
