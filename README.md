# Zano - A Node.js-like Runtime in Rust

<div align="center">

**A high-performance, Node.js-compatible runtime built in Rust with JavaScript-like syntax**

[![Crates.io](https://img.shields.io/crates/v/zano.svg)](https://crates.io/crates/zano)
[![Documentation](https://docs.rs/zano/badge.svg)](https://docs.rs/zano)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[Installation](#installation) •
[Quick Start](#quick-start) •
[Examples](#examples) •
[Built-in Modules](#built-in-modules) •
[Documentation](#documentation)

</div>

---

## Why Zano?

Zano brings the familiar Node.js development experience to the Rust ecosystem, offering:

- **Blazing Fast**: Built on Tokio's async runtime
- **Memory Safe**: All the safety guarantees of Rust
- **Familiar Syntax**: Write JavaScript-like code that just works
- **Package Management**: Full package.json support
- **Built-in Modules**: File system, HTTP, and more out of the box

## Installation

### Install from Crates.io
```bash
cargo install zano
```

### Build from Source
```bash
git clone https://github.com/sazalo101/zano
cd zano
cargo build --release
cargo install --path .
```

## Quick Start

Create a new Zano project:
```bash
# Initialize a new project
zano init

# Create your first script
echo 'console.log("Hello, Zano!")' > hello.zn

# Run it
zano hello.zn
```

## Basic Syntax

Zano supports JavaScript-like syntax with powerful features:

```javascript
// Variables and functions
let name = "World"
const greeting = "Hello"

function greet(target) {
    return greeting + ", " + target + "!"
}

console.log(greet(name))  // Output: Hello, World!

// Arrays and Objects
let numbers = [1, 2, 3, 4, 5]
let person = {
    name: "Alice",
    age: 30
}

console.log("First number:", numbers[0])
console.log("Person:", person.name)

// Control Flow
if (person.age >= 18) {
    console.log("Adult")
} else {
    console.log("Minor")
}

// Loops
let count = 0
while (count < 3) {
    console.log("Count:", count)
    count = count + 1
}
```

## Built-in Modules

### Console Module
```javascript
console.log("Info message")
console.error("Error message")  
console.warn("Warning message")
```

### File System Module
```javascript
const fs = require('fs')

// Write and read files
try {
    fs.writeFile('data.txt', 'Hello from Zano!')
    let content = fs.readFile('data.txt')
    console.log("Content:", content)
    
    let exists = fs.exists('data.txt')
    console.log("File exists:", exists)
} catch (error) {
    console.error("File operation failed:", error)
}
```

### HTTP Module
```javascript
const http = require('http')

// Make HTTP requests
try {
    let response = http.request('https://api.github.com/users/octocat')
    console.log("Response:", response)
} catch (error) {
    console.error("Request failed:", error)
}

// Create HTTP server (basic implementation)
let server = http.createServer()
console.log("Server created:", server)
```

### Path Module
```javascript
const path = require('path')

let fullPath = path.join('home', 'user', 'documents', 'file.txt')
console.log("Full path:", fullPath)

let directory = path.dirname('/home/user/file.txt')
let filename = path.basename('/home/user/file.txt')

console.log("Directory:", directory)  // /home/user
console.log("Filename:", filename)   // file.txt
```

## Error Handling

Robust error handling with try/catch:

```javascript
function riskyOperation() {
    throw "Something went wrong!"
}

try {
    console.log("Attempting risky operation...")
    riskyOperation()
    console.log("This won't be reached")
} catch (error) {
    console.error("Caught error:", error)
} 

console.log("Program continues...")
```

## Package Management

Zano includes a built-in package manager similar to npm:

### Initialize a Project
```bash
zano init
```

Creates a `package.json`:
```json
{
  "name": "my-zano-app",
  "version": "1.0.0",
  "main": "index.zn",
  "scripts": {
    "start": "zano index.zn",
    "dev": "zano --watch index.zn"
  },
  "dependencies": {},
  "license": "MIT"
}
```

### Manage Dependencies
```bash
# Install a package
zano install package-name

# Install all dependencies
zano install

# Run scripts
zano run start
zano run dev
```

## Complete Examples

### Example 1: Simple Web API Simulation
```javascript
// api-server.zn
console.log("Starting Zano API Server")

let user1 = {id: 1, name: "Alice", email: "alice@example.com"}
let user2 = {id: 2, name: "Bob", email: "bob@example.com"}

function handleRequest(method, url) {
    console.log("Request:", method, url)
    
    if (url == "/users") {
        return {
            status: 200,
            message: "Users found"
        }
    } else {
        return {
            status: 404,
            error: "Not Found"
        }
    }
}

// Simulate API calls
let response1 = handleRequest("GET", "/users")
console.log("API Response status:", response1.status)
console.log("API Response message:", response1.message)

let response2 = handleRequest("GET", "/posts")
console.log("API Response status:", response2.status)
console.log("API Response error:", response2.error)

console.log("Server simulation complete")
```

### Example 2: File Processing Pipeline
```javascript
// file-processor.zn
const fs = require('fs')

console.log("File Processing Pipeline")

function processData(data) {
    return "Processed: " + data
}

try {
    // Create sample data
    let sampleData = "Hello World"
    fs.writeFile('input.txt', sampleData)
    console.log("Created input file")
    
    // Process the data
    let content = fs.readFile('input.txt')
    let processed = processData(content)
    
    // Save results
    fs.writeFile('output.txt', processed)
    console.log("Processed and saved results")
    
    // Verify results
    let result = fs.readFile('output.txt')
    console.log("Final result:")
    console.log(result)
    
} catch (error) {
    console.error("Processing failed:", error)
}
```

### Example 3: Data Analysis Script
```javascript
// analytics.zn
console.log("Zano Analytics Dashboard")

let jan = {month: "Jan", sales: 1000, costs: 800}
let feb = {month: "Feb", sales: 1200, costs: 900}
let mar = {month: "Mar", sales: 1500, costs: 1000}

function calculateProfit(record) {
    let profit = record.sales - record.costs
    return {month: record.month, profit: profit}
}

function processMonth(data) {
    console.log("Month:", data.month)
    console.log("  Profit: $" + data.profit)
    return data.profit
}

let totalProfit = 0
let bestProfit = 0
let bestMonth = ""

console.log("MONTHLY ANALYSIS")

// Process January
let janAnalysis = calculateProfit(jan)
let janProfit = processMonth(janAnalysis)
totalProfit = totalProfit + janProfit
if (janProfit > bestProfit) {
    bestProfit = janProfit
    bestMonth = janAnalysis.month
}

// Process February  
let febAnalysis = calculateProfit(feb)
let febProfit = processMonth(febAnalysis)
totalProfit = totalProfit + febProfit
if (febProfit > bestProfit) {
    bestProfit = febProfit
    bestMonth = febAnalysis.month
}

// Process March
let marAnalysis = calculateProfit(mar)
let marProfit = processMonth(marAnalysis)
totalProfit = totalProfit + marProfit  
if (marProfit > bestProfit) {
    bestProfit = marProfit
    bestMonth = marAnalysis.month
}

console.log("SUMMARY")
console.log("Total Profit: $" + totalProfit)
console.log("Best Month:", bestMonth, "($" + bestProfit + ")")
```

## CLI Commands

```bash
# Run Zano scripts
zano script.zn
zano path/to/script.zn

# Evaluate code directly
zano -e "console.log('Quick test')"

# Interactive REPL
zano -i

# Package management
zano init                    # Initialize new project
zano install [package]      # Install dependencies  
zano run <script>           # Run package.json scripts

# Get help
zano --help
```

## Language Features

| Feature | Status | Example |
|---------|--------|---------|
| Variables | Available | `let x = 5; const y = "hello"` |
| Functions | Available | `function add(a, b) { return a + b }` |
| Arrays | Available | `let arr = [1, 2, 3]; arr[0]` |
| Objects | Available | `let obj = {name: "test"}; obj.name` |
| Control Flow | Available | `if/else`, `while` loops |
| Error Handling | Available | `try/catch/throw` |
| Modules | Available | `const fs = require('fs')` |
| Async/Await | Planned | Coming soon |
| Classes | Planned | Coming soon |
| Destructuring | Planned | Coming soon |

## Architecture

Zano is built on these core technologies:

- **Parser**: Custom JavaScript-compatible lexer and parser
- **Runtime**: Tokio-based async execution engine
- **Memory**: Rust's ownership system ensures memory safety
- **Modules**: Pluggable module system with built-in and custom modules
- **Package Manager**: Cargo-inspired dependency management

## Performance

Zano leverages Rust's performance characteristics:

- **Zero-cost abstractions**: No runtime overhead
- **Memory efficient**: No garbage collector needed
- **Concurrent**: Built on Tokio for handling thousands of concurrent operations
- **Fast startup**: Compiled binary starts instantly

## Development

### Building from Source
```bash
git clone https://github.com/sazalo101/zano
cd zano
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Contributing
1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run `cargo test` and `cargo fmt`
5. Submit a pull request

## Comparison with Node.js

| Aspect | Zano | Node.js |
|--------|------|---------|
| **Runtime** | Tokio (Rust) | libuv (C++) |
| **Memory Safety** | Compile-time | Runtime errors possible |
| **Performance** | Very High | High |
| **Startup Time** | Instant | ~50ms |
| **Memory Usage** | Minimal | Higher baseline |
| **Ecosystem** | Growing | Massive |
| **Syntax** | JS-compatible | JavaScript |
| **Error Messages** | Precise | Sometimes cryptic |

## Roadmap

### Version 1.1.0
- [ ] Full async/await support
- [ ] HTTP server implementation
- [ ] Module bundling system
- [ ] Package registry integration

### Version 1.2.0
- [ ] Class syntax support
- [ ] Destructuring assignment
- [ ] Template literals
- [ ] JSON parsing utilities

### Version 2.0.0
- [ ] TypeScript-like static typing
- [ ] WebAssembly module support
- [ ] Built-in testing framework
- [ ] Debug tooling

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Community

- **Issues**: [GitHub Issues](https://github.com/sazalo101/zano/issues)
- **Discussions**: [GitHub Discussions](https://github.com/sazalo101/zano/discussions)
- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md)

## Star History

If you find Zano useful, please consider giving it a star on GitHub!

---

<div align="center">

**Built with love in Rust**

[Back to top](#zano---a-nodejs-like-runtime-in-rust)

</div>
