//! Rich terminal viewer app for lex
pub mod app;
pub mod fileviewer;
pub mod model;
pub mod treeviewer;
pub mod ui;
#[allow(clippy::module_inception)]
pub mod viewer;

#[cfg(test)]
pub mod tests;
