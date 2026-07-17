const MAX_OWNER_BYTES: usize = 96;
const MAX_CONNECTIONS: u16 = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageAccessProfileV1 {
    ReadWrite,
    ReadOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageNamespaceRequestV1 {
    owner: String,
    required: bool,
    profile: StorageAccessProfileV1,
    client_connections: u16,
}

impl StorageNamespaceRequestV1 {
    pub fn new(
        owner: String,
        required: bool,
        profile: StorageAccessProfileV1,
        client_connections: u16,
    ) -> Result<Self, StorageRequestErrorV1> {
        if owner.is_empty()
            || owner.len() > MAX_OWNER_BYTES
            || !owner
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(StorageRequestErrorV1::Owner);
        }
        if client_connections == 0 || client_connections > MAX_CONNECTIONS {
            return Err(StorageRequestErrorV1::Connections);
        }
        Ok(Self {
            owner,
            required,
            profile,
            client_connections,
        })
    }
    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub const fn required(&self) -> bool {
        self.required
    }
    pub const fn profile(&self) -> StorageAccessProfileV1 {
        self.profile
    }
    pub const fn client_connections(&self) -> u16 {
        self.client_connections
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRequestErrorV1 {
    Owner,
    Connections,
}
