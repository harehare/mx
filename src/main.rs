//! mx - Markdown-based task runner CLI

use clap::{Parser, Subcommand};
use colored::*;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use mx::{Config, ExecutionMode, Runner};

const DEFAULT_TASKS_FILE: &str = "README.md";

#[derive(Parser)]
#[command(name = "mx")]
#[command(about = "Markdown-based task runner", long_about = None)]
#[command(version)]
struct Cli {
    /// Task name to execute (shorthand for 'run' command)
    #[arg(value_name = "TASK")]
    task: Option<String>,

    /// Path to the markdown file
    #[arg(short, long, default_value = DEFAULT_TASKS_FILE)]
    file: PathBuf,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Heading level for sections (1-6)
    #[arg(short, long)]
    level: Option<u8>,

    /// Override runtime for a language (format: lang:command, e.g., python:python3.11)
    #[arg(short, long, value_name = "LANG:COMMAND")]
    runtime: Vec<String>,

    /// Set execution mode for runtime overrides (stdin, file, arg)
    #[arg(short, long, value_name = "MODE")]
    execution_mode: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a task from a markdown file
    Run {
        /// Task name (section title) to execute
        task: String,

        /// Path to the markdown file
        #[arg(short, long, default_value = DEFAULT_TASKS_FILE)]
        file: PathBuf,

        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Heading level for sections (1-6)
        #[arg(short, long)]
        level: Option<u8>,

        /// Override runtime for a language (format: lang:command, e.g., python:python3.11)
        #[arg(short, long, value_name = "LANG:COMMAND")]
        runtime: Vec<String>,

        /// Set execution mode for runtime overrides (stdin, file, arg)
        #[arg(short, long, value_name = "MODE")]
        execution_mode: Option<String>,
    },

    /// List all available tasks in a markdown file
    List {
        /// Path to the markdown file
        #[arg(short, long, default_value = DEFAULT_TASKS_FILE)]
        file: PathBuf,

        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Heading level for sections (1-6)
        #[arg(short, long)]
        level: Option<u8>,
    },

    /// Generate a sample configuration file
    Init {
        /// Output path for configuration file
        #[arg(short, long, default_value = "mx.toml")]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run {
            file,
            task,
            config,
            level,
            runtime,
            execution_mode,
        }) => run_task(file, task, config, level, runtime, execution_mode)?,
        Some(Commands::List {
            file,
            config,
            level,
        }) => list_tasks(file, config, level)?,
        Some(Commands::Init { output }) => init_config(output)?,
        None => {
            // If no subcommand, check if task is provided
            if let Some(task) = cli.task {
                run_task(cli.file, task, cli.config, cli.level, cli.runtime, cli.execution_mode)?;
            } else {
                // No task provided, list available tasks
                list_tasks(cli.file, cli.config, cli.level)?;
            }
        }
    }

    Ok(())
}

/// Run a specific task
fn run_task(
    markdown_path: PathBuf,
    task_name: String,
    config_path: Option<PathBuf>,
    level: Option<u8>,
    runtime_overrides: Vec<String>,
    execution_mode: Option<String>,
) -> Result<()> {
    let mut config = load_config(config_path)?;

    // Override heading level if specified
    if let Some(level) = level {
        config.heading_level = level;
    }

    // Parse execution mode if specified
    let exec_mode = if let Some(mode_str) = execution_mode {
        Some(ExecutionMode::try_from(mode_str.as_str()).into_diagnostic()?)
    } else {
        None
    };

    // Apply runtime overrides
    if !runtime_overrides.is_empty() {
        config
            .apply_runtime_overrides(&runtime_overrides, exec_mode)
            .into_diagnostic()?;
    }

    let mut runner = Runner::new(config);

    println!("Running task: {}", task_name);
    println!();

    runner
        .run_task(&markdown_path, &task_name)
        .into_diagnostic()?;

    Ok(())
}

/// List all available tasks
fn list_tasks(
    markdown_path: PathBuf,
    config_path: Option<PathBuf>,
    level: Option<u8>,
) -> Result<()> {
    let mut config = load_config(config_path)?;

    // Override heading level if specified
    if let Some(level) = level {
        config.heading_level = level;
    }

    let mut runner = Runner::new(config);

    let sections = runner.list_task_sections(&markdown_path).into_diagnostic()?;

    if sections.is_empty() {
        println!(
            "{}",
            format!("No tasks found in {}", markdown_path.display()).yellow()
        );
        return Ok(());
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{} {}\n\n",
        "Available tasks in".bold(),
        markdown_path.display().to_string().cyan()
    ));

    for section in sections {
        if let Some(desc) = section.description {
            let trimmed = desc.trim();
            if !trimmed.is_empty() {
                output.push_str(&format!(
                    "  {} {}\n",
                    section.title.green().bold(),
                    format!("- {}", trimmed).bright_black()
                ));
            } else {
                output.push_str(&format!("  {}\n", section.title.green().bold()));
            }
        } else {
            output.push_str(&format!("  {}\n", section.title.green().bold()));
        }
    }

    print!("{}", output);

    Ok(())
}

/// Initialize configuration file
fn init_config(output_path: PathBuf) -> Result<()> {
    if output_path.exists() {
        return Err(miette::miette!(
            "Configuration file already exists: {}",
            output_path.display()
        ));
    }

    let config = Config::default();
    let toml = toml::to_string_pretty(&config).into_diagnostic()?;

    std::fs::write(&output_path, toml).into_diagnostic()?;
    println!("Configuration file created: {}", output_path.display());

    Ok(())
}

/// Load configuration from file or use default
fn load_config(config_path: Option<PathBuf>) -> Result<Config> {
    let config = if let Some(path) = config_path {
        Config::from_file(&path).into_diagnostic()?
    } else {
        Config::default()
    };

    Ok(config)
}
