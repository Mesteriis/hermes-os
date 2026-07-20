//! Desired Event Hub topology derived from the approved catalog.

pub(crate) mod plan;
mod scheduler_dispatches;
mod scheduler_receipts;
pub(crate) mod subject;

pub use plan::{EventConsumerPlanV1, EventPublisherPermitPlanV1, EventTopologyPlanV1};
pub(crate) use scheduler_dispatches::{
    SchedulerDispatchTopologyErrorV1, scheduler_dispatch_bindings,
};
pub(crate) use scheduler_receipts::{SchedulerReceiptTopologyErrorV1, scheduler_receipt_bindings};

use super::catalog::EventCatalogContractV1;

pub fn plan(
    contracts: &[EventCatalogContractV1],
    configuration: &hermes_kernel_control_store::PlatformEventHubTopologyV1,
) -> Result<EventTopologyPlanV1, String> {
    EventTopologyPlanV1::from_contracts(contracts, configuration)
}
