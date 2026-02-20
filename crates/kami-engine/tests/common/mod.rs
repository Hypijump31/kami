//! Shared fixtures for kami-engine integration tests.

/// Minimal WASM component that echoes its input as Ok(input).
///
/// Canonical ABI for `result<string, string>`:
/// - Core params: (ptr: i32, len: i32) for input string
/// - Core result: (i32) pointer to return-area struct
///   - [retptr+0]: i32 discriminant (0=Ok, 1=Err)
///   - [retptr+4]: i32 string ptr
///   - [retptr+8]: i32 string len
pub const ECHO_COMPONENT_WAT: &str = r#"
(component
  (core module $m
    (memory (export "memory") 1)

    ;; Return area at a fixed location (offset 0x1000)
    (global $retarea (mut i32) (i32.const 4096))

    (func (export "cabi_realloc") (param i32 i32 i32 i32) (result i32)
      ;; Bump allocator starting at offset 256
      ;; new_size is param 3, we just return a fixed offset above static data
      i32.const 256
    )

    ;; run(ptr: i32, len: i32) -> i32 (retptr)
    (func (export "run") (param $ptr i32) (param $len i32) (result i32)
      ;; Write discriminant 0 (Ok) at retarea+0
      global.get $retarea
      i32.const 0
      i32.store

      ;; Write string ptr at retarea+4
      global.get $retarea
      i32.const 4
      i32.add
      local.get $ptr
      i32.store

      ;; Write string len at retarea+8
      global.get $retarea
      i32.const 8
      i32.add
      local.get $len
      i32.store

      ;; Return pointer to the result struct
      global.get $retarea
    )

    ;; post-return cleanup (takes the retptr)
    (func (export "cabi_post_run") (param i32))
  )

  (core instance $i (instantiate $m))

  (func (export "run")
    (param "input" string)
    (result (result string (error string)))
    (canon lift
      (core func $i "run")
      (memory $i "memory")
      (realloc (func $i "cabi_realloc"))
      (post-return (func $i "cabi_post_run"))
    )
  )
)
"#;
