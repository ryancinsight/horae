//! Hierarchical integration harness for the Horae time-integration contracts.
//!
//! The leaf modules retain their original contract assertions untouched;
//! one Cargo target replaces the previous flat target-per-file topology
//! (12 binaries linking the provider per build). Nextest still isolates per
//! test, and the committed timeout/serial-group profile applies unchanged:
//! test-name filters match the trailing test name with or without the
//! harness module prefix.

mod adaptive_loop;
mod allocation;
mod embedded;
mod fixed_step;
mod implicit_step;
mod implicit_system;
mod layout;
mod multi_step_march;
mod order_of_accuracy;
mod policy;
mod properties;
mod robertson;
