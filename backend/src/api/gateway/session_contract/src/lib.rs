//! Browser-session authority contract shared by Kernel and Core Gateway.

mod browser;

pub use browser::client_bootstrap::{
    ClientBootstrapAuthority, ClientBootstrapProjectionV1, ClientModuleProjectionV1,
    ClientModuleSettingsProjectionV1, ClientSettingValueEntryV1, ClientSettingValueV1,
    ClientSurfaceAvailabilityProjectionV1, ClientSurfaceAvailabilityStateV1, ClientSurfaceIdV1,
    ClientSystemComponentIdV1, ClientSystemComponentStateV1,
    ClientSystemComponentStatusProjectionV1,
};
pub use browser::enrollment::{
    BrowserEnrollmentAuthority, BrowserEnrollmentInputV1, BrowserEnrollmentV1,
};
pub use browser::identity::{
    BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
    BrowserDeviceCredentialV1, BrowserDevicePrincipalV1, BrowserPairingAuthority,
    GatewayIdentityFenceV1,
};

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_rp_id(value: &str) -> bool {
    value == "localhost"
        || (value.len() <= 253
            && value.split('.').count() >= 2
            && value.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                    && !label.starts_with('-')
                    && !label.ends_with('-')
            }))
}
