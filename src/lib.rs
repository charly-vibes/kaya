//! kaya — a living record of exploration.
//!
//! The library surface of the kaya research compiler: the genesis-style CLI
//! envelope, the `kaya.event/v0` record, the append-only ledger, the research
//! graph, capture adapters, documentation projections, and the promotion
//! gate.

pub mod adapters;
pub mod envelope;
pub mod event;
pub mod ledger;
pub mod model;
pub mod projection;
pub mod promotion;
pub mod publication;
pub mod verification;
