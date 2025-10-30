#[path = "viewer/app.rs"]
pub mod app;
#[path = "viewer/model.rs"]
pub mod model;
#[path = "viewer/ui.rs"]
pub mod ui;
#[allow(clippy::module_inception)]
#[path = "viewer/viewer.rs"]
pub mod viewer;
#[path = "viewer/viewer_main.rs"]
pub mod viewer_main;

#[cfg(test)]
#[path = "viewer/tests.rs"]
pub mod tests;

#[allow(dead_code)]
fn main() {}
