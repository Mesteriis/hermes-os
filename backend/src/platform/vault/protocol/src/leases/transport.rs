//! Compact deterministic encoding for a lease issue request inside HPKE plaintext.

use crate::{LeaseAudienceV1, VaultActionV1, VaultProtocolError, VaultPurposeRequestV1};

pub(super) fn write_audience(bytes: &mut Vec<u8>, audience: &LeaseAudienceV1) {
    write_text(bytes, audience.module_registration_id());
    write_text(bytes, audience.runtime_instance_id());
    bytes.extend_from_slice(&audience.runtime_generation().to_le_bytes());
    bytes.extend_from_slice(&audience.grant_epoch().to_le_bytes());
}

pub(super) fn write_purpose(bytes: &mut Vec<u8>, purpose: &VaultPurposeRequestV1) {
    write_text(bytes, purpose.purpose_id());
    write_text(bytes, purpose.configuration_instance_id());
    bytes.push(purpose.allowed_secret_classes().len() as u8);
    bytes.extend(
        purpose
            .allowed_secret_classes()
            .iter()
            .map(|class| class.code() as u8),
    );
    bytes.push(purpose.actions().len() as u8);
    bytes.extend(purpose.actions().iter().map(|action| action.code() as u8));
    bytes.extend_from_slice(&purpose.requested_lease_ttl_seconds().to_le_bytes());
}

pub(super) fn write_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.push(value.len() as u8);
    bytes.extend_from_slice(value.as_bytes());
}

pub(super) struct TransportReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> TransportReader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(super) fn text(&mut self) -> Result<String, VaultProtocolError> {
        let length = usize::from(self.byte()?);
        let value = self.take(length)?;
        std::str::from_utf8(value)
            .map(str::to_owned)
            .map_err(|_| VaultProtocolError::InvalidPurpose)
    }

    pub(super) fn u64(&mut self) -> Result<u64, VaultProtocolError> {
        self.take(8)
            .and_then(|value| {
                value
                    .try_into()
                    .map_err(|_| VaultProtocolError::InvalidPurpose)
            })
            .map(u64::from_le_bytes)
    }

    pub(super) fn purpose(&mut self) -> Result<VaultPurposeRequestV1, VaultProtocolError> {
        let purpose_id = self.text()?;
        let configuration_instance_id = self.text()?;
        let allowed_secret_classes = self.secret_classes()?;
        let actions = self.actions()?;
        let requested_lease_ttl_seconds = self
            .take(4)
            .and_then(|value| {
                value
                    .try_into()
                    .map_err(|_| VaultProtocolError::InvalidPurpose)
            })
            .map(u32::from_le_bytes)?;
        VaultPurposeRequestV1::new(
            purpose_id,
            configuration_instance_id,
            allowed_secret_classes,
            actions,
            requested_lease_ttl_seconds,
        )
    }

    pub(super) fn audience(&mut self) -> Result<LeaseAudienceV1, VaultProtocolError> {
        LeaseAudienceV1::new(self.text()?, self.text()?, self.u64()?, self.u64()?)
    }

    fn secret_classes(&mut self) -> Result<Vec<crate::SecretClassV1>, VaultProtocolError> {
        self.values(|value| crate::SecretClassV1::from_code(i64::from(value)))
    }

    fn actions(&mut self) -> Result<Vec<VaultActionV1>, VaultProtocolError> {
        self.values(|value| VaultActionV1::from_code(i64::from(value)))
    }

    fn values<T>(
        &mut self,
        decode: impl Fn(u8) -> Option<T>,
    ) -> Result<Vec<T>, VaultProtocolError> {
        let count = usize::from(self.byte()?);
        self.take(count)?
            .iter()
            .map(|value| decode(*value).ok_or(VaultProtocolError::InvalidPurpose))
            .collect()
    }

    fn byte(&mut self) -> Result<u8, VaultProtocolError> {
        self.take(1).map(|value| value[0])
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], VaultProtocolError> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(VaultProtocolError::InvalidPurpose)?;
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    pub(super) const fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
