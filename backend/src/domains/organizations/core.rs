mod aliases;
mod contact_links;
mod departments;
mod domains;
mod errors;
mod evidence;
mod identity;
mod related;

pub use aliases::{OrgAliasStore, OrganizationAlias};
pub use contact_links::{OrgContactLink, OrgContactLinkStore};
pub use departments::{OrgDepartment, OrgDepartmentStore};
pub use domains::{OrgDomainStore, OrganizationDomain};
pub use errors::OrgCoreError;
pub(crate) use evidence::{
    link_email_domain_projection_in_transaction, link_entity_in_transaction,
    link_organization_in_transaction, link_review_transition_in_transaction,
};
pub use identity::{OrgIdentityStore, OrganizationIdentity};
pub use related::{RelatedOrgStore, RelatedOrganization};
