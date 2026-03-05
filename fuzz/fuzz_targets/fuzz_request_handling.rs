//! Fuzz target for request handling logic.
//!
//! Uses arbitrary to generate structured Request values and verify handling doesn't panic.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

/// Fuzzable subset of Request that doesn't require server state.
#[derive(Debug, Arbitrary)]
enum FuzzRequest {
    Spawn {
        cmd: Vec<String>,
        rows: u16,
        cols: u16,
    },
    Kill {
        id: String,
        signal: i32,
    },
    Send {
        id: String,
        data: String,
        newline: bool,
    },
    SendBytes {
        id: String,
        data: Vec<u8>,
    },
}

fuzz_target!(|req: FuzzRequest| {
    // Convert to real Request and serialize/deserialize
    let request = match req {
        FuzzRequest::Spawn { cmd, rows, cols } => {
            vessel::protocol::Request::Spawn {
                cmd,
                rows,
                cols,
                name: None,
                labels: vec![],
                timeout: None,
                max_output: None,
                env: vec![],
                cwd: None,
                no_resize: false,
                record: false,
            }
        }
        FuzzRequest::Kill { id, signal } => vessel::protocol::Request::Kill {
            id: Some(id),
            labels: vec![],
            all: false,
            signal,
            proc_filter: None,
        },
        FuzzRequest::Send { id, data, newline } => {
            vessel::protocol::Request::Send { id, data, newline }
        }
        FuzzRequest::SendBytes { id, data } => vessel::protocol::Request::SendBytes { id, data },
    };

    // Roundtrip through JSON - should not panic
    if let Ok(json) = serde_json::to_string(&request) {
        let _ = serde_json::from_str::<vessel::protocol::Request>(&json);
    }
});
