//! Core types and desync strategy definitions for Discord DPI bypass.

pub mod desync;
pub mod packet;
pub mod strategy;
pub mod tls;

pub use desync::{DesyncEngine, DesyncTargetFilter, ProcessOutcome};
pub use strategy::{DesyncMethod, DesyncParams, Profile, Stage};
