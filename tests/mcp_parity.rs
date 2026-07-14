//! # MCP/CLI Parity Test
//!
//! Regression test for the critical finding in `laverna-fixes-v2.md`: the
//! MCP `validate` tool handler used a *different* code path than the CLI
//! `validate` subcommand, so a fix landed in the human-facing CLI but not in
//! the path an LLM actually calls (`tools/call`). This test pipes JSON-RPC
//! into the built binary and asserts the `validate` result matches the CLI
//! path, so the divergence class of bug gets caught by CI.
//!
//! Run: `cargo test --test mcp_parity -- --nocapture --features mcp`
#![cfg(feature = "mcp")]

use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

/// Expressions that must be rejected (passed == false) and accepted (passed == true).
const MUST_FAIL: &[&str] = &[
    "2 + 3 = 6",
    "1 = 2",
    "7*7 = 0",
    "sqrt(-1) = sqrt(-1)",
    "1e400 = 1e400",
    "0.1 + 0.2 = 0.4",
];

const MUST_PASS: &[&str] = &["2 + 3 = 5", "0.1 + 0.2 = 0.3", "5 = 5"];

/// Spawn the `laverna mcp` binary and send a sequence of JSON-RPC lines,
/// returning the raw stdout captured.
fn run_mcp_session(lines: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_laverna");
    let mut child = Command::new(bin)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn laverna mcp");

    {
        let stdin = child.stdin.as_mut().unwrap();
        for line in lines {
            writeln!(stdin, "{line}").unwrap();
        }
    }
    // Explicitly close stdin so the child gets EOF and exits.
    child.stdin.take();

    let stdout = child.stdout.as_mut().unwrap();
    let reader = std::io::BufReader::new(stdout);
    let mut out = String::new();
    for line in reader.lines() {
        out.push_str(&line.unwrap());
        out.push('\n');
    }
    let _ = child.wait();
    out
}

/// Parse the `passed:` field from a `validate` tool text response.
fn extract_passed(text: &str) -> Option<bool> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("passed:") {
            return match rest.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

/// Extract the text content from a JSON-RPC tools/call response.
fn extract_tool_text(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v["result"]["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c["text"].as_str())
        .map(|s| s.to_string())
}

#[test]
fn mcp_validate_agrees_with_cli_validate() {
    // Build the JSON-RPC session: initialize, then one validate call per expr.
    let mut lines: Vec<String> =
        vec![r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string()];
    for (i, expr) in MUST_FAIL.iter().chain(MUST_PASS.iter()).enumerate() {
        let id = i + 2;
        lines.push(format!(
            r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"validate","arguments":{{"expression":"{expr}"}}}}}}"#
        ));
    }

    let lines_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let output = run_mcp_session(&lines_refs);

    // Parse each response by its id.
    let mut responses: std::collections::HashMap<u64, String> = std::collections::HashMap::new();
    for line in output.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(id) = v["id"].as_u64() {
                if let Some(text) = extract_tool_text(line) {
                    responses.insert(id, text);
                }
            }
        }
    }

    let exprs: Vec<&str> = MUST_FAIL.iter().chain(MUST_PASS.iter()).copied().collect();
    let mut id = 2;
    for expr in &exprs {
        let text = responses
            .get(&id)
            .unwrap_or_else(|| panic!("no MCP response for validate({expr:?})"));
        let passed = extract_passed(text)
            .unwrap_or_else(|| panic!("no 'passed:' in MCP response for {expr:?}: {text}"));
        let expected = MUST_FAIL.contains(expr) == false; // false for MUST_FAIL, true for MUST_PASS
        assert_eq!(
            passed, expected,
            "MCP validate diverged from CLI on {expr:?}: got passed={passed}, expected {expected}"
        );
        id += 1;
    }
}

/// §5: MCP `validate` must include diagnostic *strings*, not just the count.
#[test]
fn mcp_validate_includes_diagnostic_text() {
    let lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"validate","arguments":{"expression":"2 + 3 = 6"}}}"#,
    ];
    let output = run_mcp_session(&lines);

    let mut text: Option<String> = None;
    for line in output.lines() {
        if let Some(t) = extract_tool_text(line) {
            text = Some(t);
        }
    }
    let text = text.expect("no MCP validate response text");

    // The message "Equation does not balance: 5 != 6" must appear in the MCP output,
    // not just "diagnostics: 2".
    assert!(
        text.contains("Equation does not balance: 5 != 6"),
        "MCP validate response missing diagnostic string:\n{text}"
    );
    assert_eq!(extract_passed(&text), Some(false));
}
