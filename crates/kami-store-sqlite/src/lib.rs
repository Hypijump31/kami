//! # kami-store-sqlite
//!
//! SQLite adapter for the KAMI tool registry.
//! Implements `ToolRepository` with full CRUD operations.

pub mod migrations;
mod query_builder;
pub mod repository;
mod repository_impl;
mod row_mapping;

pub use repository::SqliteToolRepository;
