//! dirtidy - A directory organization and cleanup utility
//!
//! This library provides utilities for detecting file types, categorizing files,
//! organizing directories by file type, and undoing those operations.

pub mod cli;
pub mod file_category;
pub mod file_organizer;
pub mod undo;

pub use file_category::{Category, FileMapper};
pub use file_organizer::FileOrganizer;
pub use undo::{UndoManager, UndoReport};

pub use cli::{OrganizeCommand, run_cli};
