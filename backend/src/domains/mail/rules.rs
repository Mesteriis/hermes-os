mod errors;
mod evaluation;
mod mode;
mod models;
mod rows;
mod store;
mod validation;

#[cfg(test)]
mod tests;

pub use errors::EmailRuleError;
pub use mode::RuleMode;
pub use models::{EmailRule, NewEmailRule, RuleAction, RuleMatchResult};
pub use store::EmailRuleStore;
