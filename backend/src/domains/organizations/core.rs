mod aliases;
mod departments;
mod domains;
mod errors;
mod evidence;
mod identity;
mod persona_links;
mod related;

pub use aliases::{OrgAliasStore, OrganizationAlias};
pub use departments::{OrgDepartment, OrgDepartmentStore};
pub use domains::{OrgDomainStore, OrganizationDomain};
pub use errors::OrgCoreError;
pub(crate) use evidence::{
    link_email_domain_projection_in_transaction, link_entity_in_transaction,
    link_organization_in_transaction, link_review_transition_in_transaction,
};
pub use identity::{OrgIdentityStore, OrganizationIdentity};
pub use persona_links::OrgPersonaLinkStore as OrganizationPersonaLinkPort;
pub use persona_links::{OrgPersonaLink, OrgPersonaLinkStore};
pub use related::{RelatedOrgStore, RelatedOrganization};
