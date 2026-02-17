-- Initial schema for KAMI tool registry.

CREATE TABLE IF NOT EXISTS tools (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    wasm_path   TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    manifest    TEXT NOT NULL,  -- JSON blob of full ToolManifest
    install_path TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tools_name ON tools(name);
CREATE INDEX IF NOT EXISTS idx_tools_enabled ON tools(enabled);
