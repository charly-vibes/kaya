//! kaya — a living record of exploration.
//!
//! The library surface of the kaya research compiler. This MVP exposes only
//! the genesis-style CLI envelope; the event, ledger, model, promotion,
//! adapter, and projection modules arrive with later milestones.

pub mod adapters;
pub mod envelope;
pub mod event;
pub mod ledger;
pub mod model;
pub mod projection;
pub mod promotion;
