//! Sanitized receipt returned by write-only owner provisioning commands.

use crate::VaultActionV1;

const RECEIPT_MAJOR: u8 = 1;
const RECEIPT_BYTES: usize = 27;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultProvisioningStateV1 {
    Active,
    Retired,
    Deleted,
}

impl VaultProvisioningStateV1 {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Active => 1,
            Self::Retired => 2,
            Self::Deleted => 3,
        }
    }

    pub const fn from_code(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Active),
            2 => Some(Self::Retired),
            3 => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultProvisioningReceiptV1 {
    operation_id: [u8; 16],
    action: VaultActionV1,
    secret_revision: u64,
    state: VaultProvisioningStateV1,
}

impl VaultProvisioningReceiptV1 {
    pub fn new(
        operation_id: [u8; 16],
        action: VaultActionV1,
        secret_revision: u64,
        state: VaultProvisioningStateV1,
    ) -> Result<Self, VaultProvisioningReceiptError> {
        if operation_id == [0; 16]
            || secret_revision == 0
            || !matches!(
                action,
                VaultActionV1::Create
                    | VaultActionV1::ReplaceCas
                    | VaultActionV1::Retire
                    | VaultActionV1::Delete
            )
            || state != state_for_action(action)
        {
            return Err(VaultProvisioningReceiptError::Malformed);
        }
        Ok(Self {
            operation_id,
            action,
            secret_revision,
            state,
        })
    }

    #[must_use]
    pub const fn operation_id(&self) -> &[u8; 16] {
        &self.operation_id
    }

    #[must_use]
    pub const fn action(&self) -> VaultActionV1 {
        self.action
    }

    #[must_use]
    pub const fn secret_revision(&self) -> u64 {
        self.secret_revision
    }

    #[must_use]
    pub const fn state(&self) -> VaultProvisioningStateV1 {
        self.state
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(RECEIPT_BYTES);
        bytes.push(RECEIPT_MAJOR);
        bytes.extend_from_slice(&self.operation_id);
        bytes.push(self.action.code() as u8);
        bytes.extend_from_slice(&self.secret_revision.to_be_bytes());
        bytes.push(self.state.code());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, VaultProvisioningReceiptError> {
        if bytes.len() != RECEIPT_BYTES || bytes.first() != Some(&RECEIPT_MAJOR) {
            return Err(VaultProvisioningReceiptError::Malformed);
        }
        let operation_id = bytes[1..17]
            .try_into()
            .map_err(|_| VaultProvisioningReceiptError::Malformed)?;
        let action = VaultActionV1::from_code(i64::from(bytes[17]))
            .ok_or(VaultProvisioningReceiptError::Malformed)?;
        let secret_revision = u64::from_be_bytes(
            bytes[18..26]
                .try_into()
                .map_err(|_| VaultProvisioningReceiptError::Malformed)?,
        );
        let state = VaultProvisioningStateV1::from_code(bytes[26])
            .ok_or(VaultProvisioningReceiptError::Malformed)?;
        Self::new(operation_id, action, secret_revision, state)
    }
}

#[must_use]
pub const fn state_for_action(action: VaultActionV1) -> VaultProvisioningStateV1 {
    match action {
        VaultActionV1::Retire => VaultProvisioningStateV1::Retired,
        VaultActionV1::Delete => VaultProvisioningStateV1::Deleted,
        _ => VaultProvisioningStateV1::Active,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultProvisioningReceiptError {
    Malformed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_receipt_round_trips_without_secret_carriers() {
        let receipt = VaultProvisioningReceiptV1::new(
            [7; 16],
            VaultActionV1::ReplaceCas,
            2,
            VaultProvisioningStateV1::Active,
        )
        .expect("valid receipt");

        assert_eq!(
            VaultProvisioningReceiptV1::decode(&receipt.encode()),
            Ok(receipt)
        );
    }
}
