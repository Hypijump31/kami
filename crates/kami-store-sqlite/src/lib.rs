//! # kami-store-sqlite
//!
//! SQLite adapter for the KAMI tool registry.
//! Implements `ToolRepository` with full CRUD operations.

pub mod migrations;
pub mod repository;

pub use repository::SqliteToolRepository;
