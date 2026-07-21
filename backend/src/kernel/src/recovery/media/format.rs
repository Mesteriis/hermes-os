//! Canonical binary format for signed whole-instance recovery media.

use std::path::{Component, PathBuf};

const MANIFEST_MAGIC: &[u8; 8] = b"HRMEDIA4";
const MAX_ENTRIES: usize = 256;
const MAX_PATH_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(crate) enum RecoveryMediaComponentV1 {
    ControlStore = 1,
    Vault = 2,
    Storage = 3,
    Blob = 4,
    EventHub = 5,
    Scheduler = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum RecoveryMediaInclusionV1 {
    Required = 1,
    Conditional = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryMediaInventoryV1 {
    blob_enabled: bool,
    scheduler_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryMediaEntryV1 {
    component: RecoveryMediaComponentV1,
    inclusion: RecoveryMediaInclusionV1,
    path: String,
    size_bytes: u64,
    sha256: [u8; 32],
}

pub(crate) struct RecoveryMediaManifestV1 {
    provenance: RecoveryMediaProvenanceV1,
    inventory: RecoveryMediaInventoryV1,
    entries: Vec<RecoveryMediaEntryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryMediaProvenanceV1 {
    backup_generation: u64,
    source_commit: String,
    cargo_lock_sha256: [u8; 32],
    toolchain_sha256: [u8; 32],
    policy_sha256: [u8; 32],
}

impl RecoveryMediaInventoryV1 {
    pub(crate) fn new(blob_enabled: bool, scheduler_enabled: bool) -> Self {
        Self {
            blob_enabled,
            scheduler_enabled,
        }
    }

    pub(crate) fn blob_enabled(self) -> bool {
        self.blob_enabled
    }

    pub(crate) fn scheduler_enabled(self) -> bool {
        self.scheduler_enabled
    }

    fn encode(self) -> u8 {
        u8::from(self.blob_enabled) | (u8::from(self.scheduler_enabled) << 1)
    }

    fn decode(value: u8) -> Result<Self, String> {
        if value & !0b11 != 0 {
            return Err(invalid_manifest());
        }
        Ok(Self::new(value & 1 != 0, value & 2 != 0))
    }
}

impl RecoveryMediaManifestV1 {
    pub(crate) fn encode(
        provenance: RecoveryMediaProvenanceV1,
        inventory: RecoveryMediaInventoryV1,
        entries: Vec<RecoveryMediaEntryV1>,
    ) -> Result<Vec<u8>, String> {
        provenance.validate()?;
        validate_entries(inventory, &entries)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MANIFEST_MAGIC);
        bytes.extend_from_slice(&provenance.backup_generation.to_be_bytes());
        bytes.push(provenance.source_commit.len() as u8);
        bytes.extend_from_slice(provenance.source_commit.as_bytes());
        bytes.extend_from_slice(&provenance.cargo_lock_sha256);
        bytes.extend_from_slice(&provenance.toolchain_sha256);
        bytes.extend_from_slice(&provenance.policy_sha256);
        bytes.push(inventory.encode());
        bytes.extend_from_slice(&(entries.len() as u16).to_be_bytes());
        for entry in entries {
            bytes.push(entry.component as u8);
            bytes.push(entry.inclusion as u8);
            let path = entry.path.as_bytes();
            bytes.extend_from_slice(&(path.len() as u16).to_be_bytes());
            bytes.extend_from_slice(path);
            bytes.extend_from_slice(&entry.size_bytes.to_be_bytes());
            bytes.extend_from_slice(&entry.sha256);
        }
        Ok(bytes)
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(bytes);
        if cursor.take(8)? != MANIFEST_MAGIC {
            return Err(invalid_manifest());
        }
        let provenance = RecoveryMediaProvenanceV1::decode(&mut cursor)?;
        let inventory = RecoveryMediaInventoryV1::decode(cursor.byte()?)?;
        let count = usize::from(u16::from_be_bytes(cursor.array()?));
        if count == 0 || count > MAX_ENTRIES {
            return Err(invalid_manifest());
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(decode_entry(&mut cursor)?);
        }
        if !cursor.remaining().is_empty() {
            return Err(invalid_manifest());
        }
        validate_entries(inventory, &entries)?;
        Ok(Self {
            provenance,
            inventory,
            entries,
        })
    }

    pub(crate) fn inventory(&self) -> RecoveryMediaInventoryV1 {
        self.inventory
    }

    pub(super) fn entries(&self) -> &[RecoveryMediaEntryV1] {
        &self.entries
    }
}

impl RecoveryMediaEntryV1 {
    pub(crate) fn new(
        component: RecoveryMediaComponentV1,
        inclusion: RecoveryMediaInclusionV1,
        path: String,
        size_bytes: u64,
        sha256: [u8; 32],
    ) -> Result<Self, String> {
        if inclusion != component.inclusion() || !component.owns_path(&path) {
            return Err("recovery media component classification is invalid".to_owned());
        }
        Ok(Self {
            component,
            inclusion,
            path,
            size_bytes,
            sha256,
        })
    }

    pub(super) fn path(&self) -> &str {
        &self.path
    }

    pub(super) fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub(super) fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    pub(crate) fn canonical_order(&self) -> (u8, &str) {
        (self.component as u8, &self.path)
    }
}

impl RecoveryMediaComponentV1 {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::ControlStore),
            2 => Ok(Self::Vault),
            3 => Ok(Self::Storage),
            4 => Ok(Self::Blob),
            5 => Ok(Self::EventHub),
            6 => Ok(Self::Scheduler),
            _ => Err(invalid_manifest()),
        }
    }

    pub(crate) fn inclusion(self) -> RecoveryMediaInclusionV1 {
        match self {
            Self::Blob | Self::Scheduler => RecoveryMediaInclusionV1::Conditional,
            Self::ControlStore | Self::Vault | Self::Storage | Self::EventHub => {
                RecoveryMediaInclusionV1::Required
            }
        }
    }

    pub(crate) fn from_path(path: &str) -> Result<Self, String> {
        match PathBuf::from(path).components().next() {
            Some(Component::Normal(prefix)) if prefix == "control-store" => Ok(Self::ControlStore),
            Some(Component::Normal(prefix)) if prefix == "vault" => Ok(Self::Vault),
            Some(Component::Normal(prefix)) if prefix == "storage" => Ok(Self::Storage),
            Some(Component::Normal(prefix)) if prefix == "blob" => Ok(Self::Blob),
            Some(Component::Normal(prefix)) if prefix == "event-hub" => Ok(Self::EventHub),
            Some(Component::Normal(prefix)) if prefix == "scheduler" => Ok(Self::Scheduler),
            _ => Err("recovery media component path is invalid".to_owned()),
        }
    }

    fn owns_path(self, path: &str) -> bool {
        valid_relative_path(path)
            && path.starts_with(match self {
                Self::ControlStore => "control-store/",
                Self::Vault => "vault/",
                Self::Storage => "storage/",
                Self::Blob => "blob/",
                Self::EventHub => "event-hub/",
                Self::Scheduler => "scheduler/",
            })
    }
}

impl RecoveryMediaInclusionV1 {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Required),
            2 => Ok(Self::Conditional),
            _ => Err(invalid_manifest()),
        }
    }
}

impl RecoveryMediaProvenanceV1 {
    pub(crate) fn new(
        backup_generation: u64,
        source_commit: String,
        cargo_lock_sha256: [u8; 32],
        toolchain_sha256: [u8; 32],
        policy_sha256: [u8; 32],
    ) -> Result<Self, String> {
        let value = Self {
            backup_generation,
            source_commit,
            cargo_lock_sha256,
            toolchain_sha256,
            policy_sha256,
        };
        value.validate()?;
        Ok(value)
    }

    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, String> {
        let backup_generation = u64::from_be_bytes(cursor.array()?);
        let commit_length = usize::from(cursor.byte()?);
        let source_commit = std::str::from_utf8(cursor.take(commit_length)?)
            .map_err(|_| invalid_manifest())?
            .to_owned();
        Self::new(
            backup_generation,
            source_commit,
            cursor.array()?,
            cursor.array()?,
            cursor.array()?,
        )
        .map_err(|_| invalid_manifest())
    }

    fn validate(&self) -> Result<(), String> {
        let commit = self.source_commit.as_bytes();
        if self.backup_generation == 0
            || !(commit.len() == 40 || commit.len() == 64)
            || !commit.iter().all(u8::is_ascii_hexdigit)
        {
            return Err("recovery media provenance is invalid".to_owned());
        }
        Ok(())
    }
}

fn decode_entry(cursor: &mut Cursor<'_>) -> Result<RecoveryMediaEntryV1, String> {
    let component = RecoveryMediaComponentV1::decode(cursor.byte()?)?;
    let inclusion = RecoveryMediaInclusionV1::decode(cursor.byte()?)?;
    let length = usize::from(u16::from_be_bytes(cursor.array()?));
    if length == 0 || length > MAX_PATH_BYTES {
        return Err(invalid_manifest());
    }
    let path = std::str::from_utf8(cursor.take(length)?)
        .map_err(|_| invalid_manifest())?
        .to_owned();
    RecoveryMediaEntryV1::new(
        component,
        inclusion,
        path,
        u64::from_be_bytes(cursor.array()?),
        cursor.array()?,
    )
    .map_err(|_| invalid_manifest())
}

fn validate_entries(
    inventory: RecoveryMediaInventoryV1,
    entries: &[RecoveryMediaEntryV1],
) -> Result<(), String> {
    if entries.is_empty() || entries.len() > MAX_ENTRIES {
        return Err(invalid_manifest());
    }
    if entries.windows(2).any(|pair| {
        (pair[0].component, pair[0].path.as_str()) >= (pair[1].component, pair[1].path.as_str())
    }) {
        return Err("recovery media manifest entries are not canonical".to_owned());
    }
    let has = |component| entries.iter().any(|entry| entry.component == component);
    let exact = has(RecoveryMediaComponentV1::ControlStore)
        && has(RecoveryMediaComponentV1::Vault)
        && has(RecoveryMediaComponentV1::Storage)
        && has(RecoveryMediaComponentV1::EventHub)
        && has(RecoveryMediaComponentV1::Blob) == inventory.blob_enabled
        && has(RecoveryMediaComponentV1::Scheduler) == inventory.scheduler_enabled;
    exact.then_some(()).ok_or_else(|| {
        "recovery media component inventory does not match the signed plan".to_owned()
    })
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(invalid_manifest)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(invalid_manifest)?;
        self.position = end;
        Ok(bytes)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.take(N)?.try_into().map_err(|_| invalid_manifest())
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && PathBuf::from(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn invalid_manifest() -> String {
    "recovery media manifest is invalid".to_owned()
}
