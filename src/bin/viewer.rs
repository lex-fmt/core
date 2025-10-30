//! Rich terminal viewer app for txxt
#[path = "viewer/app.rs"]
pub mod app;
#[path = "viewer/fileviewer.rs"]
pub mod fileviewer;
#[path = "viewer/model.rs"]
pub mod model;
#[path = "viewer/treeviewer.rs"]
pub mod treeviewer;
#[path = "viewer/ui.rs"]
pub mod ui;
#[allow(clippy::module_inception)]
#[path = "viewer/viewer.rs"]
pub mod viewer;

#[cfg(test)]
#[path = "viewer/tests.rs"]
pub mod tests;

#[allow(dead_code)]
fn main() {}
