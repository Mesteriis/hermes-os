//! Exact public Connect route declared by an owner capability descriptor.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleClientRpcRouteV1 {
    registration_id: String,
    capability_id: String,
    owner: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    path: String,
}

impl ModuleClientRpcRouteV1 {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>, capability_id: impl Into<String>, owner: impl Into<String>,
        contract_name: impl Into<String>, contract_major: u32, contract_revision: u32,
        contract_schema_sha256: [u8; 32], path: impl Into<String>,
    ) -> Self {
        Self { registration_id: registration_id.into(), capability_id: capability_id.into(), owner: owner.into(), contract_name: contract_name.into(), contract_major, contract_revision, contract_schema_sha256, path: path.into() }
    }
    #[must_use] pub fn registration_id(&self) -> &str { &self.registration_id }
    #[must_use] pub fn capability_id(&self) -> &str { &self.capability_id }
    #[must_use] pub fn owner(&self) -> &str { &self.owner }
    #[must_use] pub fn contract_name(&self) -> &str { &self.contract_name }
    #[must_use] pub const fn contract_major(&self) -> u32 { self.contract_major }
    #[must_use] pub const fn contract_revision(&self) -> u32 { self.contract_revision }
    #[must_use] pub const fn contract_schema_sha256(&self) -> &[u8; 32] { &self.contract_schema_sha256 }
    #[must_use] pub fn path(&self) -> &str { &self.path }
}
