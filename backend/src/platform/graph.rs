use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Person,
    EmailAddress,
    Message,
    Document,
    Project,
    Organization,
    Task,
    Event,
    Decision,
    Obligation,
    Knowledge,
}

impl GraphNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::EmailAddress => "email_address",
            Self::Message => "message",
            Self::Document => "document",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Task => "task",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Knowledge => "knowledge",
        }
    }
}

pub fn node_id(kind: GraphNodeKind, stable_key: &str) -> String {
    format!("graph:node:v1:{}:{stable_key}", kind.as_str())
}
