//! 🐝 Beambin!
#![doc = include_str!("../../../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/trisuaso/beambin/issues")]
pub mod api;
pub mod config;
pub mod database;
pub mod model;

pub use databeam::DatabaseOpts;
