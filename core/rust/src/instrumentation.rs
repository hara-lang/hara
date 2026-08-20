//! Native event, inspection, and execution-control contracts.
//!
//! The hub is owned by a Hara [`crate::Runtime`]. It registers trusted
//! instruments and real execution targets without exposing either target
//! implementation or an ambient Hara-level authority. Evaluator and HBC probe
//! integration is intentionally delivered in the next instrumentation tranche.

mod hub;
mod model;

pub use hub::{
    ControlLease, InstrumentationAttachment, InstrumentationError,
    InstrumentationHub, SessionCleanup,
};
pub use model::{
    Capability, EventDelivery, EventEnvelope, EventKind, EventLocation,
    EventMask, EventPhase, InstrumentDirective, InstrumentFilter,
    InstrumentHandle, InstrumentMode, InstrumentRegistration, ProjectionLimits,
    ProjectionRequest, RuntimeBackend, SourceSpan, TargetDescriptor,
    TargetHandle, TargetKind, INSTRUMENTATION_EVENT_SCHEMA,
    INSTRUMENTATION_PROTOCOL,
};

#[cfg(test)]
mod tests;
