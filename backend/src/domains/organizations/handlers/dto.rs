use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct OrgList {
    pub items: Vec<crate::domains::organizations::api::Organization>,
}
#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
pub struct NewOrg {
    pub name: String,
    pub kind: Option<String>,
}
#[derive(Deserialize)]
pub struct NewIdentity {
    pub identity_type: String,
    pub identity_value: String,
    pub source: Option<String>,
}
#[derive(Deserialize)]
pub struct NewAlias {
    pub alias: String,
}
#[derive(Deserialize)]
pub struct NewDepartment {
    pub name: String,
}
#[derive(Deserialize)]
pub struct LinkContact {
    pub person_id: String,
    pub role: Option<String>,
}
