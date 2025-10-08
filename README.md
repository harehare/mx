<h1 align="center">mx</h1>

Markdown Task Runner

[![ci](https://github.com/harehare/mx/actions/workflows/ci.yml/badge.svg)](https://github.com/harehare/mx/actions/workflows/ci.yml)

`mx` is a task runner that executes code blocks in Markdown files based on section titles.
It is implemented using [mq](https://github.com/harehare/mq), a jq-like command-line tool for Markdown processing, to parse and extract sections from Markdown documents.

![demo](assets/demo.gif)

> [!WARNING]
> `mx` is currently under active development.

## Features

- Execute code blocks from specific sections in Markdown files
- Configurable runtimes for different programming languages
- Support for custom heading levels
- TOML-based configuration
- Built on top of the mq query language

## Installation

### Cargo

```sh
$ cargo install --git https://github.com/harehare/mx.git
```

## Usage

### Run a task (shorthand)

```bash
# Run from README.md (default)
mx "Task Name"

# Run from a specific file
mx -f tasks.md "Task Name"
```

### Run a task (explicit)

```bash
mx run "Task Name"
mx run --file tasks.md "Task Name"
```

### List available tasks

```bash
# List tasks from README.md (default)
mx

# List tasks from a specific file
mx -f tasks.md
mx list --file tasks.md
```

### Initialize configuration

```bash
mx init
```

This creates an `mx.toml` file with default runtime settings.

## Configuration

Create an `mx.toml` file to customize runtime behavior:

```toml
# Heading level for sections (default: 2, i.e., ## headings)
heading_level = 2

[runtimes]
bash = "bash"
sh = "sh"
python = "python3"
ruby = "ruby"
node = "node"
javascript = "node"
js = "node"
go = "go run"
php = "php"
perl = "perl"
```

```bash
# Using shorthand (from tasks.md by default)
mx Build

# From a specific file
mx -f tasks.md Build

# Using explicit run command
mx run Build
mx run --file tasks.md Build
```

## Example

---

## Bash

```bash
echo "Hello, world!"
```

## Python

```python
print("Hello, world!")
```

## Ruby

```ruby
puts "Hello, world!"
```

## JavaScript

```javascript
console.log("Hello, world!");
```

## Go

```go
package main
import "fmt"
func main() {
    fmt.Println("Hello, world!")
}
```

## Rust

```rust
fn main() {
    println!("Hello, world!");
}
```

## mq

```mq
print("Hello, world!")
```

## License

MIT
