use hermes_hub_backend::domains::mail::core::{
    CommunicationProviderKind, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::secrets::SecretKind;

#[test]
fn telegram_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("telegram_user").expect("telegram user"),
        CommunicationProviderKind::TelegramUser
    );
    assert_eq!(
        CommunicationProviderKind::try_from("telegram_bot").expect("telegram bot"),
        CommunicationProviderKind::TelegramBot
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramApiHash.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramBotToken.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramSessionKey
            .accepts_secret_kind(SecretKind::PrivateKey)
    );
    assert!(
        !ProviderAccountSecretPurpose::TelegramBotToken.accepts_secret_kind(SecretKind::Password)
    );
}
