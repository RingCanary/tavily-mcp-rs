use crate::types::{
    CallToolParams, CallToolRequest, CallToolResponse, CallToolResult, Content, ListToolsRequest,
    ListToolsResponse, ListToolsResult, McpError, Tool,
};
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, error};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

// Define the MCP Server trait
#[async_trait]
pub trait McpServer {
    async fn handle_list_tools(&self) -> Result<Vec<Tool>, McpError>;
    async fn handle_call_tool(&self, params: CallToolParams) -> Result<CallToolResult, McpError>;
}

// Define the MCP Server transport
pub struct StdioServerTransport;

impl StdioServerTransport {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn process_requests<S: McpServer + Send + Sync>(
        &self,
        server: &S,
    ) -> Result<(), McpError> {
        let (tx, mut rx) = mpsc::channel::<String>(100);
        
        // Spawn a task to read from stdin
        let read_task = tokio::spawn(async move {
            let mut reader = BufReader::new(tokio::io::stdin());
            let mut line = String::new();
            
            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break;
                }
                
                if let Err(e) = tx.send(line.clone()).await {
                    error!("Failed to send line to channel: {}", e);
                    break;
                }
                
                line.clear();
            }
        });
        
        // Process incoming requests
        while let Some(line) = rx.recv().await {
            if line.trim().is_empty() {
                continue;
            }
            
            debug!("Received request: {}", line);
            
            let response = match serde_json::from_str::<Value>(&line) {
                Ok(value) => {
                    let method = value["method"].as_str().unwrap_or_default();
                    
                    match method {
                        "list_tools" => {
                            let request: ListToolsRequest = serde_json::from_value(value)?;
                            let tools = server.handle_list_tools().await?;
                            
                            let response = ListToolsResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: ListToolsResult { tools },
                            };
                            
                            serde_json::to_string(&response)?
                        }
                        "call_tool" => {
                            let request: CallToolRequest = serde_json::from_value(value)?;
                            
                            match server.handle_call_tool(request.params).await {
                                Ok(result) => {
                                    let response = CallToolResponse {
                                        jsonrpc: "2.0".to_string(),
                                        id: request.id,
                                        result,
                                    };
                                    
                                    serde_json::to_string(&response)?
                                }
                                Err(e) => {
                                    let error_message = format!("Error: {}", e);
                                    let response = json!({
                                        "jsonrpc": "2.0",
                                        "id": request.id,
                                        "result": {
                                            "content": [{
                                                "type": "text",
                                                "text": error_message
                                            }],
                                            "is_error": true
                                        }
                                    });
                                    
                                    serde_json::to_string(&response)?
                                }
                            }
                        }
                        _ => {
                            let error_message = format!("Unknown method: {}", method);
                            let response = json!({
                                "jsonrpc": "2.0",
                                "id": value["id"].as_str().unwrap_or("0"),
                                "error": {
                                    "code": -32601,
                                    "message": error_message
                                }
                            });
                            
                            serde_json::to_string(&response)?
                        }
                    }
                }
                Err(e) => {
                    let error_message = format!("Invalid JSON: {}", e);
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": error_message
                        }
                    });
                    
                    serde_json::to_string(&response)?
                }
            };
            
            debug!("Sending response: {}", response);
            
            let mut stdout = tokio::io::stdout();
            stdout.write_all(response.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
        
        read_task.await.map_err(|e| McpError::General(e.to_string()))?;
        
        Ok(())
    }
}

// Helper function to format results
pub fn format_results(response: &crate::types::TavilyResponse) -> String {
    let mut output = Vec::new();
    
    // Include answer if available
    if let Some(answer) = &response.answer {
        output.push(format!("Answer: {}", answer));
        output.push("\nSources:".to_string());
        
        for result in &response.results {
            output.push(format!("- {}: {}", result.title, result.url));
        }
        
        output.push("".to_string());
    }
    
    // Format detailed search results
    output.push("Detailed Results:".to_string());
    
    for result in &response.results {
        output.push(format!("\nTitle: {}", result.title));
        output.push(format!("URL: {}", result.url));
        output.push(format!("Content: {}", result.content));
        
        if let Some(raw_content) = &result.raw_content {
            output.push(format!("Raw Content: {}", raw_content));
        }
    }
    
    output.join("\n")
}
