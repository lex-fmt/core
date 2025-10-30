//! Viewer trait and event types
//!
//! The Viewer trait defines a common interface for UI components that:
//! - Render themselves given a model and area
//! - Handle keyboard input and return events
//!
//! This abstraction allows different viewers (FileViewer, TreeViewer) to
//! be treated uniformly by the main App.

use super::model::{Model, NodeId};
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Events that can be emitted by viewers
///
/// These represent model changes that should be applied after handling input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ViewerEvent {
    /// Select a tree node
    SelectNode(NodeId),
    /// Select a text position
    SelectPosition(usize, usize),
    /// Toggle whether a node is expanded
    ToggleNodeExpansion(NodeId),
    /// No change to model
    NoChange,
}

/// Trait for UI viewers
///
/// A viewer is a component that:
/// - Knows how to render itself given a model
/// - Knows how to interpret keyboard input
/// - Emits ViewerEvents when user interactions require model changes
pub trait Viewer {
    /// Render this viewer to the given area
    fn render(&self, frame: &mut Frame, area: Rect, model: &Model);

    /// Handle a keyboard event and return the resulting event
    fn handle_key(&mut self, key: KeyEvent, model: &Model) -> Option<ViewerEvent>;
}
