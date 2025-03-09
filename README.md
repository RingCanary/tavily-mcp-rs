# Tavily MCP Server - Rust Implementation 

> **Compatible with both [Cursor](https://cursor.sh) and [Claude Desktop](https://claude.ai/desktop)!**
>
> Tavily MCP is also compatible with any MCP client

The Model Context Protocol (MCP) is an open standard that enables AI systems to interact seamlessly with various data sources and tools, facilitating secure, two-way connections.

Developed by Anthropic, the Model Context Protocol (MCP) enables AI assistants like Claude to seamlessly integrate with Tavily's advanced search and data extraction capabilities. This integration provides AI models with real-time access to web information, complete with sophisticated filtering options and domain-specific search features.

This is a Rust implementation of the Tavily MCP server, which provides:
- Seamless interaction with the tavily-search and tavily-extract tools
- Real-time web search capabilities through the tavily-search tool
- Intelligent data extraction from web pages via the tavily-extract tool

## Prerequisites 

Before you begin, ensure you have:

- [Tavily API key](https://app.tavily.com/home)
  - If you don't have a Tavily API key, you can sign up for a free account [here](https://app.tavily.com/home)
- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or higher)
  - You can verify your Rust installation by running:
    - `rustc --version`
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (comes with Rust)
  - You can verify your Cargo installation by running:
    - `cargo --version`

## Installation 

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/your-username/tavily-mcp-rs.git
cd tavily-mcp-rs
```

2. Build the project:
```bash
cargo build --release
```

3. Set up your environment variables:
```bash
# Create a .env file with your Tavily API key
echo "TAVILY_API_KEY=your_api_key_here" > .env
```

4. Run the server:
```bash
cargo run --release
```

## Testing the MCP Server 

There are several ways to test the Tavily MCP server without using Claude Desktop or Cursor:

### 1. Using the Built-in Test Client

This repository includes a test client that can be used to test the MCP server directly from the command line:

```bash
# Run the test client
cargo run --bin test_client
```

The test client will:
1. Start the MCP server as a child process
2. Send a `list_tools` request to get the available tools
3. Send a `call_tool` request to perform a search using the tavily-search tool
4. Display the responses from the server

### 2. Manual Testing with JSON-RPC

You can also test the MCP server by manually sending JSON-RPC requests to it using standard input/output:

1. Start the MCP server in one terminal:
```bash
cargo run
```

2. In another terminal, use tools like `echo` to send JSON-RPC requests via standard input:
```bash
# List available tools
echo '{"jsonrpc": "2.0", "method": "list_tools", "id": "1"}' | cargo run

# Call the tavily-search tool
echo '{"jsonrpc": "2.0", "method": "call_tool", "id": "2", "params": {"name": "tavily-search", "arguments": {"query": "What is Rust programming language?"}}}' | cargo run
```

Alternatively, you can create a simple file with your request and pipe it to the server:
```bash
# Create a request file
cat > request.json << EOF
{"jsonrpc": "2.0", "method": "list_tools", "id": "1"}
EOF

# Send the request to the server
cat request.json | cargo run
```

### 3. Creating a Python Test Client

You can also create a simple Python script to test the MCP server:

```python
import json
import subprocess
import sys

def main():
    # Start the MCP server
    process = subprocess.Popen(
        ["cargo", "run", "--quiet", "--bin", "tavily-mcp-rs"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    
    # Test 1: List Tools
    list_tools_request = {
        "jsonrpc": "2.0",
        "method": "list_tools",
        "id": "1"
    }
    
    # Send the request
    process.stdin.write(json.dumps(list_tools_request) + "\n")
    process.stdin.flush()
    
    # Read the response
    response = process.stdout.readline()
    response_json = json.loads(response)
    print("List Tools Response:")
    print(json.dumps(response_json, indent=2))
    
    # Terminate the process
    process.terminate()
    process.wait()

if __name__ == "__main__":
    main()
```

## Configuring MCP Clients 

### Configuring Cursor 

> **Note**: Requires Cursor version 0.45.6 or higher

To set up the Tavily MCP server in Cursor:

1. Open Cursor Settings
2. Navigate to Features > MCP Servers
3. Click on the "+ Add New MCP Server" button
4. Fill out the following information:
   - **Name**: Enter a nickname for the server (e.g., "tavily-mcp-rs")
   - **Type**: Select "command" as the type
   - **Command**: Enter the command to run the server:
     ```bash
     env TAVILY_API_KEY=your-api-key /path/to/tavily-mcp-rs/target/release/tavily-mcp-rs
     ```
     > **Important**: Replace `your-api-key` with your Tavily API key. You can get one at [app.tavily.com/home](https://app.tavily.com/home)

### Configuring the Claude Desktop app 

#### For macOS:

```bash
# Create the config file if it doesn't exist
touch "$HOME/Library/Application Support/Claude/claude_desktop_config.json"

# Opens the config file in TextEdit 
open -e "$HOME/Library/Application Support/Claude/claude_desktop_config.json"
```

#### For Windows:
```bash
code %APPDATA%\Claude\claude_desktop_config.json
```

#### Add the Tavily server configuration:

Replace `your-api-key-here` with your actual [Tavily API key](https://tavily.com/api-keys) and `/path/to/tavily-mcp-rs` with the actual path where you built the project.

```json
{
  "mcpServers": {
    "tavily-mcp-rs": {
      "command": "/path/to/tavily-mcp-rs/target/release/tavily-mcp-rs",
      "env": {
        "TAVILY_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

## Usage in Claude Desktop App 

Once the installation is complete, and the Claude desktop app is configured, you must completely close and re-open the Claude desktop app to see the tavily-mcp server. You should see a hammer icon in the bottom left of the app, indicating available MCP tools.

Now Claude will have complete access to the tavily-mcp server, including the tavily-search and tavily-extract tools.

### Tavily Search Examples

1. **General Web Search**:
```
Can you search for recent developments in quantum computing?
```

2. **News Search**:
```
Search for news articles about AI startups from the last 7 days.
```

3. **Domain-Specific Search**:
```
Search for climate change research on nature.com and sciencedirect.com
```

### Tavily Extract Examples 

1. **Extract Article Content**:
```
Extract the main content from this article: https://example.com/article
```

## Development 

### Project Structure

- `src/main.rs` - Entry point for the application
- `src/types.rs` - Data structures and type definitions
- `src/mcp.rs` - MCP protocol implementation
- `src/tavily.rs` - Tavily API client implementation
- `src/bin/test_client.rs` - Test client for the MCP server

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Run the test client
cargo run --bin test_client
```

## License

MIT
