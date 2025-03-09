use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the MCP server as a child process
    let mut child = Command::new("cargo")
        .args(["run", "--quiet", "--bin", "tavily-mcp-rs"])
        .current_dir(".")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Get handles to child's stdin and stdout
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = io::BufReader::new(stdout);

    // Test 1: List Tools
    println!("Testing 'list_tools' request...");
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "method": "list_tools",
        "id": "1"
    });

    // Send the request
    stdin.write_all(format!("{}\n", list_tools_request).as_bytes())?;
    stdin.flush()?;

    // Read the response
    let mut response = String::new();
    reader.read_line(&mut response)?;
    
    // Parse and print the response
    let response_value: Value = serde_json::from_str(&response)?;
    println!("List Tools Response:");
    println!("{}", serde_json::to_string_pretty(&response_value)?);

    // Test 2: Call Tool (tavily-search)
    println!("\nTesting 'call_tool' request for tavily-search...");
    let call_tool_request = json!({
        "jsonrpc": "2.0",
        "method": "call_tool",
        "id": "2",
        "params": {
            "name": "tavily-search",
            "arguments": {
                "query": "What is the latest news about Rust programming language?",
                "max_results": 3
            }
        }
    });

    // Send the request
    stdin.write_all(format!("{}\n", call_tool_request).as_bytes())?;
    stdin.flush()?;

    // Read the response
    let mut response = String::new();
    reader.read_line(&mut response)?;
    
    // Parse and print the response
    let response_value: Value = serde_json::from_str(&response)?;
    println!("Call Tool Response:");
    println!("{}", serde_json::to_string_pretty(&response_value)?);

    // Terminate the child process
    drop(stdin);
    child.wait()?;

    Ok(())
}