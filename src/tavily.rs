use crate::mcp::{format_results, McpServer, StdioServerTransport};
use crate::types::{
    CallToolParams, CallToolResult, Content, ExtractParams, McpError, SearchParams, TavilyResponse, Tool,
};
use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct TavilyClient {
    api_key: String,
    client: Client,
    base_urls: HashMap<String, String>,
}

impl TavilyClient {
    pub fn new(api_key: String) -> Self {
        let mut base_urls = HashMap::new();
        base_urls.insert("search".to_string(), "https://api.tavily.com/search".to_string());
        base_urls.insert("extract".to_string(), "https://api.tavily.com/extract".to_string());

        Self {
            api_key,
            client: Client::new(),
            base_urls,
        }
    }

    pub async fn run(&self) -> Result<(), McpError> {
        info!("Starting Tavily MCP server...");
        let transport = StdioServerTransport::new();
        transport.process_requests(self).await
    }

    async fn search(&self, params: SearchParams) -> Result<TavilyResponse, McpError> {
        let mut search_params = params;
        search_params.api_key = Some(self.api_key.clone());

        // Add topic: "news" if query contains the word "news"
        if search_params.query.to_lowercase().contains("news") && search_params.topic.is_none() {
            search_params.topic = Some("news".to_string());
        }

        let endpoint = self.base_urls.get("search").unwrap();

        let response = self
            .client
            .post(endpoint)
            .json(&search_params)
            .send()
            .await
            .map_err(|e| {
                error!("Error sending search request: {}", e);
                McpError::HttpError(e)
            })?;

        match response.status().as_u16() {
            200 => {
                let tavily_response = response.json::<TavilyResponse>().await.map_err(|e| {
                    error!("Error parsing search response: {}", e);
                    McpError::HttpError(e)
                })?;
                Ok(tavily_response)
            }
            401 => Err(McpError::InvalidApiKey),
            429 => Err(McpError::UsageLimitExceeded),
            _ => {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("Tavily API error: {} - {}", status, error_text);
                Err(McpError::General(format!(
                    "Tavily API error: {} - {}",
                    status, error_text
                )))
            }
        }
    }

    async fn extract(&self, params: ExtractParams) -> Result<TavilyResponse, McpError> {
        let mut extract_params = params;
        extract_params.api_key = Some(self.api_key.clone());

        let endpoint = self.base_urls.get("extract").unwrap();

        let response = self
            .client
            .post(endpoint)
            .json(&extract_params)
            .send()
            .await
            .map_err(|e| {
                error!("Error sending extract request: {}", e);
                McpError::HttpError(e)
            })?;

        match response.status().as_u16() {
            200 => {
                let tavily_response = response.json::<TavilyResponse>().await.map_err(|e| {
                    error!("Error parsing extract response: {}", e);
                    McpError::HttpError(e)
                })?;
                Ok(tavily_response)
            }
            401 => Err(McpError::InvalidApiKey),
            429 => Err(McpError::UsageLimitExceeded),
            _ => {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("Tavily API error: {} - {}", status, error_text);
                Err(McpError::General(format!(
                    "Tavily API error: {} - {}",
                    status, error_text
                )))
            }
        }
    }
}

#[async_trait]
impl McpServer for TavilyClient {
    async fn handle_list_tools(&self) -> Result<Vec<Tool>, McpError> {
        // Define available tools: tavily-search and tavily-extract
        let tools = vec![
            Tool {
                name: "tavily-search".to_string(),
                description: "A powerful web search tool that provides comprehensive, real-time results using Tavily's AI search engine. Returns relevant web content with customizable parameters for result count, content type, and domain filtering. Ideal for gathering current information, news, and detailed web content analysis.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": { 
                            "type": "string", 
                            "description": "Search query" 
                        },
                        "search_depth": {
                            "type": "string",
                            "enum": ["basic","advanced"],
                            "description": "The depth of the search. It can be 'basic' or 'advanced'",
                            "default": "basic"
                        },
                        "topic": {
                            "type": "string",
                            "enum": ["general","news"],
                            "description": "The category of the search. This will determine which of our agents will be used for the search",
                            "default": "general"
                        },
                        "days": {
                            "type": "number",
                            "description": "The number of days back from the current date to include in the search results. This specifies the time frame of data to be retrieved. Please note that this feature is only available when using the 'news' search topic",
                            "default": 3
                        },
                        "time_range": {
                            "type": "string",
                            "description": "The time range back from the current date to include in the search results. This feature is available for both 'general' and 'news' search topics",
                            "enum": ["day", "week", "month", "year", "d", "w", "m", "y"],
                        },
                        "max_results": { 
                            "type": "number", 
                            "description": "The maximum number of search results to return",
                            "default": 10,
                            "minimum": 5,
                            "maximum": 20
                        },
                        "include_images": { 
                            "type": "boolean", 
                            "description": "Include a list of query-related images in the response",
                            "default": false,
                        },
                        "include_image_descriptions": { 
                            "type": "boolean", 
                            "description": "Include a list of query-related images and their descriptions in the response",
                            "default": false,
                        },
                        "include_raw_content": { 
                            "type": "boolean", 
                            "description": "Include the cleaned and parsed HTML content of each search result",
                            "default": false,
                        },
                        "include_domains": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "A list of domains to specifically include in the search results, if the user asks to search on specific sites set this to the domain of the site",
                            "default": []
                        },
                        "exclude_domains": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of domains to specifically exclude, if the user asks to exclude a domain set this to the domain of the site",
                            "default": []
                        }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "tavily-extract".to_string(),
                description: "A powerful web content extraction tool that retrieves and processes raw content from specified URLs, ideal for data collection, content analysis, and research tasks.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "urls": { 
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of URLs to extract content from"
                        },
                        "extract_depth": { 
                            "type": "string",
                            "enum": ["basic","advanced"],
                            "description": "Depth of extraction - 'basic' or 'advanced', if usrls are linkedin use 'advanced' or if explicitly told to use advanced",
                            "default": "basic"
                        },
                        "include_images": { 
                            "type": "boolean", 
                            "description": "Include a list of images extracted from the urls in the response",
                            "default": false,
                        }
                    },
                    "required": ["urls"]
                }),
            },
        ];

        Ok(tools)
    }

    async fn handle_call_tool(&self, params: CallToolParams) -> Result<CallToolResult, McpError> {
        let args = params.arguments.unwrap_or_default();

        match params.name.as_str() {
            "tavily-search" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::General("Missing required parameter: query".to_string()))?
                    .to_string();

                let search_params = SearchParams {
                    query,
                    search_depth: args.get("search_depth").and_then(|v| v.as_str()).map(String::from),
                    topic: args.get("topic").and_then(|v| v.as_str()).map(String::from),
                    days: args.get("days").and_then(|v| v.as_i64()).map(|v| v as i32),
                    time_range: args.get("time_range").and_then(|v| v.as_str()).map(String::from),
                    max_results: args.get("max_results").and_then(|v| v.as_i64()).map(|v| v as i32),
                    include_images: args.get("include_images").and_then(|v| v.as_bool()),
                    include_image_descriptions: args.get("include_image_descriptions").and_then(|v| v.as_bool()),
                    include_raw_content: args.get("include_raw_content").and_then(|v| v.as_bool()),
                    include_domains: args.get("include_domains").and_then(|v| {
                        v.as_array().map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(String::from))
                                .collect()
                        })
                    }),
                    exclude_domains: args.get("exclude_domains").and_then(|v| {
                        v.as_array().map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(String::from))
                                .collect()
                        })
                    }),
                    api_key: None, // Will be added in the search method
                };

                let response = self.search(search_params).await?;
                let formatted_results = format_results(&response);

                Ok(CallToolResult {
                    content: vec![Content {
                        r#type: "text".to_string(),
                        text: formatted_results,
                    }],
                    is_error: None,
                })
            }
            "tavily-extract" => {
                let urls = args
                    .get("urls")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| McpError::General("Missing required parameter: urls".to_string()))?
                    .iter()
                    .filter_map(|item| item.as_str().map(String::from))
                    .collect::<Vec<String>>();

                if urls.is_empty() {
                    return Err(McpError::General("URLs array cannot be empty".to_string()));
                }

                let extract_params = ExtractParams {
                    urls,
                    extract_depth: args.get("extract_depth").and_then(|v| v.as_str()).map(String::from),
                    include_images: args.get("include_images").and_then(|v| v.as_bool()),
                    api_key: None, // Will be added in the extract method
                };

                let response = self.extract(extract_params).await?;
                let formatted_results = format_results(&response);

                Ok(CallToolResult {
                    content: vec![Content {
                        r#type: "text".to_string(),
                        text: formatted_results,
                    }],
                    is_error: None,
                })
            }
            _ => Err(McpError::MethodNotFound(format!(
                "Unknown tool: {}",
                params.name
            ))),
        }
    }
}
