//! Atlas time-integration policy and orchestration foundation.
//!
//! Horae advances caller-owned state through borrowed slice contracts. Method
//! markers and subcycle plans are zero-sized, stage counts are const generic,
//! and a caller-owned workspace keeps stepping allocation-free.

#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

pub mod adaptive;
pub mod events;
pub mod integration;
pub mod subcycling;
pub mod system;
pub mod time;
