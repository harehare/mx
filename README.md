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

### Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/harehare/mx/refs/heads/main/bin/install.sh | bash
```

The installer will:
- Download the latest mq binary for your platform
- Install it to `~/.mx/bin/`
- Update your shell profile to add mq to your PATH

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

# Runtimes configuration
# Simple format: language = "command"
# The execution mode defaults to "stdin"
[runtimes]
bash = "bash"
sh = "sh"
python = "python3"
ruby = "ruby"
node = "node"
javascript = "node"
js = "node"
php = "php"
perl = "perl"
jq = "jq"

# Detailed format with execution mode
# Execution modes: "stdin" (default), "file", or "arg"
# - stdin: Pass code via standard input
# - file: Write code to a temporary file and pass it as an argument
# - arg: Pass code as a command-line argument

[runtimes.go]
command = "go run"
execution_mode = "file"  # Go requires file-based execution

[runtimes.golang]
command = "go run"
execution_mode = "file"

[runtimes.mq]
command = "mq"
execution_mode = "arg"  # mq uses query as argument
```

You can also mix both formats:

```toml
[runtimes]
python = "python3"  # Simple format, uses default stdin mode

[runtimes.go]       # Detailed format with custom execution mode
command = "go run"
execution_mode = "file"
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

## License

MIT
