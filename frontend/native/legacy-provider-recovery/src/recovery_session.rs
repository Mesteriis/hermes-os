use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::bundle::LegacyProviderRecoveryBundleV1;
use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};
use crate::model::{
    LegacyProviderCandidateKindV1, LegacyProviderRecoveryCandidateV1, LegacyProviderRecoveryPlanV1,
    LegacyProviderRecoverySecretPurposeV1, LegacyProviderRecoverySourceV1,
    LegacyProviderRecoveryStateV1, RECOVERY_SCHEMA_REVISION,
};
use crate::private_files::sha256_hex;
use zeroize::Zeroizing;

const MAX_SESSIONS: usize = 4;
const SESSION_TTL: Duration = Duration::from_secs(15 * 60);

pub struct LegacyProviderRecoverySessionsV1 {
    bundle: Arc<LegacyProviderRecoveryBundleV1>,
    sessions: Mutex<BTreeMap<String, Instant>>,
    generated_session_store_keys: Mutex<BTreeMap<String, Zeroizing<Vec<u8>>>>,
}

impl LegacyProviderRecoverySessionsV1 {
    pub fn new(bundle: LegacyProviderRecoveryBundleV1) -> Self {
        Self {
            bundle: Arc::new(bundle),
            sessions: Mutex::new(BTreeMap::new()),
            generated_session_store_keys: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn start(&self) -> LegacyProviderRecoveryResultV1<LegacyProviderRecoveryPlanV1> {
        self.bundle.assert_unchanged()?;
        let mut sessions = self.lock_sessions()?;
        retain_current(&mut sessions);
        if sessions.len() >= MAX_SESSIONS {
            return Err(LegacyProviderRecoveryErrorV1::CapacityExceeded);
        }
        let session_id = random_identifier()?;
        if sessions
            .insert(session_id.clone(), Instant::now())
            .is_some()
        {
            return Err(LegacyProviderRecoveryErrorV1::CapacityExceeded);
        }
        let candidates = self
            .bundle
            .sources()
            .map(|source| LegacyProviderRecoveryCandidateV1 {
                handle: source.handle().to_owned(),
                kind: source_kind(source),
                state: source_state(source),
            })
            .collect();
        Ok(LegacyProviderRecoveryPlanV1 {
            schema_revision: RECOVERY_SCHEMA_REVISION,
            session_id,
            bundle_fingerprint_sha256: self.bundle.fingerprint_sha256().to_owned(),
            counts: self.bundle.counts().clone(),
            candidates,
        })
    }

    pub fn source(
        &self,
        session_id: &str,
        handle: &str,
    ) -> LegacyProviderRecoveryResultV1<LegacyProviderRecoverySourceV1> {
        self.require_session(session_id)?;
        self.bundle.assert_unchanged()?;
        self.bundle.source(handle).cloned()
    }

    pub fn resolve_secret(
        &self,
        session_id: &str,
        handle: &str,
        purpose: LegacyProviderRecoverySecretPurposeV1,
    ) -> LegacyProviderRecoveryResultV1<zeroize::Zeroizing<Vec<u8>>> {
        self.require_session(session_id)?;
        if purpose == LegacyProviderRecoverySecretPurposeV1::GeneratedTelegramSessionStoreKey {
            if !matches!(
                self.bundle.source(handle)?,
                LegacyProviderRecoverySourceV1::TelegramUser { .. }
            ) {
                return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
            }
            let mut keys = self
                .generated_session_store_keys
                .lock()
                .map_err(|_| LegacyProviderRecoveryErrorV1::SessionUnavailable)?;
            if let Some(key) = keys.get(handle) {
                return Ok(Zeroizing::new(key.to_vec()));
            }
            let mut key = Zeroizing::new(vec![0_u8; 32]);
            getrandom::getrandom(&mut key)
                .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
            keys.insert(handle.to_owned(), Zeroizing::new(key.to_vec()));
            return Ok(key);
        }
        self.bundle.resolve_secret(handle, purpose)
    }

    pub fn cancel(&self, session_id: &str) -> LegacyProviderRecoveryResultV1<()> {
        let removed = self
            .lock_sessions()?
            .remove(session_id)
            .ok_or(LegacyProviderRecoveryErrorV1::SessionUnavailable)?;
        let _ = removed;
        Ok(())
    }

    fn require_session(&self, session_id: &str) -> LegacyProviderRecoveryResultV1<()> {
        let mut sessions = self.lock_sessions()?;
        retain_current(&mut sessions);
        let created = sessions
            .get(session_id)
            .ok_or(LegacyProviderRecoveryErrorV1::SessionUnavailable)?;
        if created.elapsed() > SESSION_TTL {
            sessions.remove(session_id);
            return Err(LegacyProviderRecoveryErrorV1::SessionUnavailable);
        }
        Ok(())
    }

    fn lock_sessions(
        &self,
    ) -> LegacyProviderRecoveryResultV1<MutexGuard<'_, BTreeMap<String, Instant>>> {
        self.sessions
            .lock()
            .map_err(|_| LegacyProviderRecoveryErrorV1::SessionUnavailable)
    }
}

fn retain_current(sessions: &mut BTreeMap<String, Instant>) {
    sessions.retain(|_, created| created.elapsed() <= SESSION_TTL);
}

fn random_identifier() -> LegacyProviderRecoveryResultV1<String> {
    let mut bytes = [0_u8; 16];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
    Ok(sha256_hex(&bytes)[..32].to_owned())
}

fn source_kind(source: &LegacyProviderRecoverySourceV1) -> LegacyProviderCandidateKindV1 {
    match source {
        LegacyProviderRecoverySourceV1::Gmail { .. } => LegacyProviderCandidateKindV1::Gmail,
        LegacyProviderRecoverySourceV1::Icloud { .. } => LegacyProviderCandidateKindV1::Icloud,
        LegacyProviderRecoverySourceV1::TelegramUser { .. } => {
            LegacyProviderCandidateKindV1::TelegramUser
        }
    }
}

fn source_state(source: &LegacyProviderRecoverySourceV1) -> LegacyProviderRecoveryStateV1 {
    match source {
        LegacyProviderRecoverySourceV1::Gmail { .. } => {
            LegacyProviderRecoveryStateV1::ReauthorizationRequired
        }
        LegacyProviderRecoverySourceV1::Icloud { .. } => {
            LegacyProviderRecoveryStateV1::ReadyToApply
        }
        LegacyProviderRecoverySourceV1::TelegramUser { .. } => {
            LegacyProviderRecoveryStateV1::QrAuthorizationRequired
        }
    }
}
