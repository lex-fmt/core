//! Heading Hierarchy Manager for Flat → Nested Conversions
//!
//! This module provides a reusable state machine for managing heading hierarchies
//! when converting from flat document formats (Markdown, HTML, LaTeX) to Lex's
//! nested session structure.
//!
//! # The Problem
//!
//! Flat formats represent document structure with heading levels:
//!
//! ```markdown
//! # Chapter 1
//! Content
//! ## Section 1.1
//! More content
//! # Chapter 2
//! ```
//!
//! Lex represents this as nested sessions. When parsing the flat format, we need
//! to track which headings are currently "open" and close parent headings when
//! encountering a new heading at the same or higher level.
//!
//! # The Solution
//!
//! `HeadingHierarchyManager` maintains a stack of open heading levels and emits
//! the appropriate `EndHeading` events when the hierarchy changes.
//!
//! # Usage Example
//!
//! ```ignore
//! use lex_babel::mappings::heading_hierarchy::HeadingHierarchyManager;
//!
//! let mut manager = HeadingHierarchyManager::new();
//! let mut events = vec![];
//!
//! // Encounter h1
//! manager.on_heading(1, &mut events); // emits StartHeading(1)
//!
//! // Encounter h2 (nested under h1)
//! manager.on_heading(2, &mut events); // emits StartHeading(2)
//!
//! // Encounter another h1 (closes h2 and previous h1)
//! manager.on_heading(1, &mut events); // emits EndHeading(2), EndHeading(1), StartHeading(1)
//!
//! // At document end
//! manager.close_all(&mut events); // emits EndHeading(1)
//! ```

use crate::ir::events::Event;

/// Manages heading hierarchy during flat-to-nested conversion
///
/// Tracks currently open heading levels and emits appropriate start/end events
/// to maintain proper nesting when converting from flat formats to Lex sessions.
#[derive(Debug)]
pub struct HeadingHierarchyManager {
    /// Stack of currently open heading levels (e.g., [1, 2] means h1 contains h2)
    stack: Vec<usize>,
}

impl HeadingHierarchyManager {
    /// Create a new heading hierarchy manager
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Process a heading at the given level
    ///
    /// This will:
    /// 1. Close any open headings at the same or deeper level
    /// 2. Emit a `StartHeading` event for the new heading
    /// 3. Track this heading as open
    ///
    /// # Arguments
    ///
    /// * `level` - The heading level (1 for h1, 2 for h2, etc.)
    /// * `events` - The event vector to append events to
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut manager = HeadingHierarchyManager::new();
    /// let mut events = vec![];
    ///
    /// manager.on_heading(1, &mut events); // Open h1
    /// manager.on_heading(2, &mut events); // Open h2 nested in h1
    /// manager.on_heading(1, &mut events); // Close h2, close h1, open new h1
    /// ```
    pub fn on_heading(&mut self, level: usize, events: &mut Vec<Event>) {
        // Close any open headings at same or deeper level
        while let Some(&stack_level) = self.stack.last() {
            if stack_level >= level {
                events.push(Event::EndHeading(stack_level));
                self.stack.pop();
            } else {
                break;
            }
        }

        // Start new heading
        events.push(Event::StartHeading(level));
        self.stack.push(level);
    }

    /// Close all remaining open headings
    ///
    /// Call this at the end of document processing to ensure all headings
    /// are properly closed.
    ///
    /// # Arguments
    ///
    /// * `events` - The event vector to append EndHeading events to
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut manager = HeadingHierarchyManager::new();
    /// let mut events = vec![Event::StartDocument];
    ///
    /// manager.on_heading(1, &mut events);
    /// manager.on_heading(2, &mut events);
    ///
    /// // At document end:
    /// manager.close_all(&mut events);
    /// events.push(Event::EndDocument);
    ///
    /// // events now contains: StartDocument, StartHeading(1), StartHeading(2),
    /// // EndHeading(2), EndHeading(1), EndDocument
    /// ```
    pub fn close_all(&mut self, events: &mut Vec<Event>) {
        while let Some(level) = self.stack.pop() {
            events.push(Event::EndHeading(level));
        }
    }

    /// Check if there are any open headings
    ///
    /// Useful for debugging or validation.
    pub fn has_open_headings(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Get the current nesting depth
    ///
    /// Returns the number of currently open headings.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

impl Default for HeadingHierarchyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_heading() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        manager.on_heading(1, &mut events);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], Event::StartHeading(1)));
        assert_eq!(manager.depth(), 1);

        manager.close_all(&mut events);
        assert_eq!(events.len(), 2);
        assert!(matches!(events[1], Event::EndHeading(1)));
        assert_eq!(manager.depth(), 0);
    }

    #[test]
    fn test_nested_headings() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        // h1 then h2 (nested)
        manager.on_heading(1, &mut events);
        manager.on_heading(2, &mut events);

        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::StartHeading(1)));
        assert!(matches!(events[1], Event::StartHeading(2)));
        assert_eq!(manager.depth(), 2);

        manager.close_all(&mut events);
        assert_eq!(events.len(), 4);
        assert!(matches!(events[2], Event::EndHeading(2)));
        assert!(matches!(events[3], Event::EndHeading(1)));
    }

    #[test]
    fn test_same_level_closes_previous() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        // h1, then another h1 (closes first)
        manager.on_heading(1, &mut events);
        manager.on_heading(1, &mut events);

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], Event::StartHeading(1)));
        assert!(matches!(events[1], Event::EndHeading(1)));
        assert!(matches!(events[2], Event::StartHeading(1)));
        assert_eq!(manager.depth(), 1);
    }

    #[test]
    fn test_higher_level_closes_nested() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        // h1, h2, h3, then back to h1 (should close h3 and h2)
        manager.on_heading(1, &mut events);
        manager.on_heading(2, &mut events);
        manager.on_heading(3, &mut events);
        manager.on_heading(1, &mut events); // Close h3, h2, h1, then open new h1

        assert_eq!(manager.depth(), 1);

        // Should have: Start(1), Start(2), Start(3), End(3), End(2), End(1), Start(1)
        assert_eq!(events.len(), 7);
        assert!(matches!(events[3], Event::EndHeading(3)));
        assert!(matches!(events[4], Event::EndHeading(2)));
        assert!(matches!(events[5], Event::EndHeading(1)));
        assert!(matches!(events[6], Event::StartHeading(1)));
    }

    #[test]
    fn test_skip_level_still_nests() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        // h1, then h4 (skipping h2 and h3)
        manager.on_heading(1, &mut events);
        manager.on_heading(4, &mut events);

        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Event::StartHeading(1)));
        assert!(matches!(events[1], Event::StartHeading(4)));
        assert_eq!(manager.depth(), 2);

        manager.close_all(&mut events);
        assert!(matches!(events[2], Event::EndHeading(4)));
        assert!(matches!(events[3], Event::EndHeading(1)));
    }

    #[test]
    fn test_complex_hierarchy() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        // h1, h2, h3, h2 (should close h3, then open new h2)
        manager.on_heading(1, &mut events);
        manager.on_heading(2, &mut events);
        manager.on_heading(3, &mut events);
        manager.on_heading(2, &mut events); // Close h3 and previous h2, open new h2

        // Stack should be [1, 2]
        assert_eq!(manager.depth(), 2);

        // Events: Start(1), Start(2), Start(3), End(3), End(2), Start(2)
        assert_eq!(events.len(), 6);
        assert!(matches!(events[3], Event::EndHeading(3)));
        assert!(matches!(events[4], Event::EndHeading(2)));
        assert!(matches!(events[5], Event::StartHeading(2)));
    }

    #[test]
    fn test_empty_manager_close_all() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        manager.close_all(&mut events);
        assert_eq!(events.len(), 0);
        assert!(!manager.has_open_headings());
    }

    #[test]
    fn test_has_open_headings() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![];

        assert!(!manager.has_open_headings());

        manager.on_heading(1, &mut events);
        assert!(manager.has_open_headings());

        manager.close_all(&mut events);
        assert!(!manager.has_open_headings());
    }

    /// Integration test: parse a realistic document structure
    #[test]
    fn test_realistic_document() {
        let mut manager = HeadingHierarchyManager::new();
        let mut events = vec![Event::StartDocument];

        // Document structure:
        // # Chapter 1
        //   ## Section 1.1
        //     ### Subsection 1.1.1
        //   ## Section 1.2
        // # Chapter 2
        //   ## Section 2.1

        manager.on_heading(1, &mut events); // Chapter 1
        manager.on_heading(2, &mut events); // Section 1.1
        manager.on_heading(3, &mut events); // Subsection 1.1.1
        manager.on_heading(2, &mut events); // Section 1.2 (closes 1.1.1, 1.1, opens new 1.2)
        manager.on_heading(1, &mut events); // Chapter 2 (closes 1.2, 1, opens new 1)
        manager.on_heading(2, &mut events); // Section 2.1

        manager.close_all(&mut events);
        events.push(Event::EndDocument);

        // Verify structure
        assert_eq!(
            events,
            vec![
                Event::StartDocument,
                Event::StartHeading(1), // Chapter 1
                Event::StartHeading(2), // Section 1.1
                Event::StartHeading(3), // Subsection 1.1.1
                Event::EndHeading(3),   // Close 1.1.1
                Event::EndHeading(2),   // Close 1.1
                Event::StartHeading(2), // Section 1.2
                Event::EndHeading(2),   // Close 1.2
                Event::EndHeading(1),   // Close Chapter 1
                Event::StartHeading(1), // Chapter 2
                Event::StartHeading(2), // Section 2.1
                Event::EndHeading(2),   // Close 2.1
                Event::EndHeading(1),   // Close Chapter 2
                Event::EndDocument,
            ]
        );
    }
}
