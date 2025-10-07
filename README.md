# mx - Markdown Task Runner

`mx` is a task runner that executes code blocks in Markdown files based on section titles.
It is implemented using [mq](https://github.com/harehare/mq), a jq-like command-line tool for Markdown processing, to parse and extract sections from Markdown documents.

> [!WARNING]
> `mx` is currently under active development.

## Features

- Execute code blocks from specific sections in Markdown files
- Configurable runtimes for different programming languages
- Support for custom heading levels
- TOML-based configuration
- Built on top of the mq query language

## Installation

```bash
cargo install --path crates/mx
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
rust = "rustc"
go = "go run"
php = "php"
perl = "perl"
```

## Example

Create a `tasks.md` file:

```markdown
# My Project Tasks

## Build

```bash
cargo build --release
```

## Test

```bash
cargo test
```

## Deploy

```bash
./deploy.sh
```
```

Run a specific task:

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
