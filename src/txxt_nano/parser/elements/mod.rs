//! Element-specific parsers
//!
//! This module contains parsing logic for individual txxt elements.
//! Each element type has its own module with its parser implementation and tests.

pub mod annotations;
pub mod definitions;
pub mod document;
pub mod foreign;
pub mod labels;
pub mod lists;
pub mod parameters;
pub mod sessions;
