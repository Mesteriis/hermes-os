use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccountContentEgressPermissions {
    pub body: bool,
    pub attachments: bool,
    pub extracted_text: bool,
}

impl AccountContentEgressPermissions {
    pub fn from_account_config(config: &Value) -> Self {
        let Some(egress) = config.get("content_egress").and_then(Value::as_object) else {
            return Self::default();
        };
        Self {
            body: egress.get("body").and_then(Value::as_bool).unwrap_or(false),
            attachments: egress
                .get("attachments")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            extracted_text: egress
                .get("extracted_text")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }
}
