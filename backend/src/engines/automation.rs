mod constants;
mod dry_run;
mod errors;
mod evidence;
mod ids;
mod models;
mod policy;
mod rows;
mod store;
mod validation;

pub use errors::AutomationError;
pub use models::{
    AutomationPolicy, AutomationPolicyScope, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
    object_from_pairs,
};
pub use store::AutomationStore;
