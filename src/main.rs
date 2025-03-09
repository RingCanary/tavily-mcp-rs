use anyhow::Result;
use dotenv::dotenv;
use log::{error, info};
use std::env;

mod mcp;
mod tavily;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init();
    
    // Check for API key
    let api_key = env::var("TAVILY_API_KEY").expect("TAVILY_API_KEY environment variable is required");
    
    info!("Starting Tavily MCP server...");
    
    // Create and run the server
    let server = tavily::TavilyClient::new(api_key);
    
    match server.run().await {
        Ok(_) => {
            info!("Tavily MCP server running on stdio");
            Ok(())
        }
        Err(e) => {
            error!("Error running Tavily MCP server: {}", e);
            Err(e.into())
        }
    }
}
