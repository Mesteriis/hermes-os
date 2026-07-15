use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatabaseReadiness {
    status: ReadinessStatus,
    message: &'static str,
}

impl DatabaseReadiness {
    pub(crate) fn ok() -> Self {
        Self {
            status: ReadinessStatus::Ok,
            message: "database is reachable",
        }
    }

    pub(crate) fn not_configured() -> Self {
        Self {
            status: ReadinessStatus::NotConfigured,
            message: "DATABASE_URL is not configured",
        }
    }

    pub(crate) fn unavailable(message: &'static str) -> Self {
        Self {
            status: ReadinessStatus::Unavailable,
            message,
        }
    }

    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn message(&self) -> &str {
        self.message
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MigrationReadiness {
    status: ReadinessStatus,
    message: &'static str,
}

impl MigrationReadiness {
    pub(crate) fn ok() -> Self {
        Self {
            status: ReadinessStatus::Ok,
            message: "required database migrations are applied",
        }
    }

    pub(crate) fn not_configured() -> Self {
        Self {
            status: ReadinessStatus::NotConfigured,
            message: "DATABASE_URL is not configured",
        }
    }

    pub(crate) fn unavailable(message: &'static str) -> Self {
        Self {
            status: ReadinessStatus::Unavailable,
            message,
        }
    }

    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn message(&self) -> &str {
        self.message
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    Ok,
    NotConfigured,
    Unavailable,
}
