//! Tests for the `kami dev` subcommands.

use super::*;

#[test]
fn rs_file_triggers_rebuild() {
    let event = Event {
        kind: EventKind::Create(notify::event::CreateKind::File),
        paths: vec![PathBuf::from("src/lib.rs")],
        attrs: Default::default(),
    };
    assert!(is_source_event(&event));
}

#[test]
fn png_file_ignored() {
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Content,
        )),
        paths: vec![PathBuf::from("assets/logo.png")],
        attrs: Default::default(),
    };
    assert!(!is_source_event(&event));
}

#[test]
fn build_wasm_fails_without_cargo_toml() {
    let tmp = std::env::temp_dir().join("kami_test_no_cargo");
    std::fs::create_dir_all(&tmp).ok();
    let result = build_wasm(&tmp, false);
    assert!(result.is_err());
    std::fs::remove_dir_all(&tmp).ok();
}
