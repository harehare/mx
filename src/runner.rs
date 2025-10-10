//! Task runner implementation

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use mq_lang::{parse_markdown_input, Engine, Ident, RuntimeValue};
use serde::{Deserialize, Serialize};

use crate::config::{Config, ExecutionMode};
use crate::error::{Error, Result};

const SECTIONS_QUERY: &str = include_str!("../sections.mq");

/// Represents a code block in a section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Language of the code block
    pub lang: String,
    /// Code content
    pub code: String,
}

/// Represents a section with its code blocks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Section {
    /// Section title
    pub title: String,
    /// Heading level
    pub level: u8,
    /// Code blocks in this section
    pub codes: Vec<CodeBlock>,
    /// Optional description extracted from the section content
    pub description: Option<String>,
}

/// Task runner that executes code blocks in Markdown sections
pub struct Runner {
    config: Config,
    engine: Engine,
}

impl Runner {
    /// Create a new Runner with the given configuration
    pub fn new(config: Config) -> Self {
        let mut engine = Engine::default();
        engine.load_builtin_module();

        Self { config, engine }
    }

    /// Create a new Runner with default configuration
    pub fn with_default_config() -> Self {
        Self::new(Config::default())
    }

    /// Load and parse a Markdown file
    pub fn load_markdown<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        fs::read_to_string(path).map_err(Error::Io)
    }

    /// Extract sections from Markdown content
    pub fn extract_sections(&mut self, markdown: &str) -> Result<Vec<Section>> {
        let input = parse_markdown_input(markdown)
            .map_err(|e| Error::Markdown(format!("Failed to parse markdown: {}", e)))?;

        let query = format!(
            "{}\n | nodes | sections_with_code({})",
            SECTIONS_QUERY, self.config.heading_level
        );

        let result = self
            .engine
            .eval(&query, input.into_iter())
            .map_err(|e| Error::Query(format!("Failed to execute query: {}", e)))?;

        let sections = self.parse_sections(result)?;

        Ok(sections)
    }

    fn parse_sections(&self, result: mq_lang::RuntimeValues) -> Result<Vec<Section>> {
        let mut sections = Vec::new();

        for value in result.into_iter() {
            if let RuntimeValue::Dict(dict) = value {
                let section = self.parse_section(&dict)?;
                sections.push(section);
            }
        }

        Ok(sections)
    }

    fn parse_section(&self, dict: &BTreeMap<Ident, RuntimeValue>) -> Result<Section> {
        let title = dict
            .get(&Ident::from("title"))
            .and_then(|v| match v {
                RuntimeValue::String(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        let level = dict
            .get(&Ident::from("level"))
            .and_then(|v| match v {
                RuntimeValue::Number(n) => Some(n.value() as u8),
                _ => None,
            })
            .unwrap_or(self.config.heading_level);

        let codes = dict
            .get(&Ident::from("codes"))
            .and_then(|v| match v {
                RuntimeValue::Array(arr) => Some(self.parse_code_blocks(arr)),
                _ => None,
            })
            .unwrap_or_else(|| Ok(Vec::new()))?;

        let description = dict.get(&Ident::from("description")).and_then(|v| match v {
            RuntimeValue::String(s) => Some(s.to_string()),
            _ => None,
        });

        Ok(Section {
            title,
            level,
            codes,
            description,
        })
    }

    fn parse_code_blocks(&self, arr: &[RuntimeValue]) -> Result<Vec<CodeBlock>> {
        let mut blocks = Vec::new();

        for item in arr {
            if let RuntimeValue::Dict(dict) = item {
                let lang = dict
                    .get(&Ident::from("lang"))
                    .and_then(|v| match v {
                        RuntimeValue::String(s) => Some(s.to_string()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let code = dict
                    .get(&Ident::from("code"))
                    .and_then(|v| match v {
                        RuntimeValue::String(s) => Some(s.to_string()),
                        _ => None,
                    })
                    .unwrap_or_default();

                blocks.push(CodeBlock { lang, code });
            }
        }

        Ok(blocks)
    }

    pub fn find_section<'a>(&self, sections: &'a [Section], title: &str) -> Option<&'a Section> {
        sections.iter().find(|s| s.title == title)
    }

    pub fn execute_section(&self, section: &Section) -> Result<()> {
        for code_block in &section.codes {
            if code_block.lang.is_empty() {
                continue;
            }

            self.execute_code(&code_block.lang, &code_block.code)?;
        }

        Ok(())
    }

    pub fn execute_code(&self, lang: &str, code: &str) -> Result<()> {
        let runtime = self
            .config
            .get_runtime(lang)
            .ok_or_else(|| Error::RuntimeNotFound(lang.to_string()))?;

        let parts: Vec<&str> = runtime.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Error::RuntimeNotFound(lang.to_string()));
        }

        // Get execution mode from config
        let execution_mode = self.config.get_execution_mode(lang);

        match execution_mode {
            ExecutionMode::File => self.execute_code_with_file(lang, code, &parts),
            ExecutionMode::Arg => self.execute_code_with_args(code, &parts),
            ExecutionMode::Stdin => self.execute_code_with_stdin(code, &parts),
        }
    }

    fn execute_code_with_stdin(&self, code: &str, parts: &[&str]) -> Result<()> {
        let cmd = parts[0];
        let args = &parts[1..];

        // Use inherit() for stdout/stderr to preserve TTY and colors
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| Error::Execution(format!("Failed to spawn process: {}", e)))?;

        // Write code to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(code.as_bytes())
                .map_err(|e| Error::Execution(format!("Failed to write to stdin: {}", e)))?;
            drop(stdin);
        }

        // Wait for completion
        let status = child
            .wait()
            .map_err(|e| Error::Execution(format!("Failed to wait for process: {}", e)))?;

        if !status.success() {
            return Err(Error::Execution("Execution failed".to_string()));
        }

        Ok(())
    }

    fn execute_code_with_args(&self, code: &str, parts: &[&str]) -> Result<()> {
        let cmd = parts[0];
        // Append code as an argument to the command
        let mut args: Vec<&str> = parts[1..].to_vec();
        args.push(code);

        // Use inherit() for stdout/stderr to preserve TTY and colors
        let mut child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| Error::Execution(format!("Failed to spawn process: {}", e)))?;

        // Write code to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(code.as_bytes())
                .map_err(|e| Error::Execution(format!("Failed to write to stdin: {}", e)))?;
            drop(stdin);
        }

        // Wait for completion
        let status = child
            .wait()
            .map_err(|e| Error::Execution(format!("Failed to wait for process: {}", e)))?;

        if !status.success() {
            return Err(Error::Execution("Execution failed".to_string()));
        }

        Ok(())
    }

    fn execute_code_with_file(&self, lang: &str, code: &str, parts: &[&str]) -> Result<()> {
        use std::env;

        // Create temporary directory
        let temp_dir = env::temp_dir();

        // Use language name as file extension, or map known languages
        let file_ext = match lang {
            "go" | "golang" => "go",
            "python" => "py",
            "ruby" => "rb",
            "javascript" | "js" => "js",
            "typescript" | "ts" => "ts",
            _ => lang, // Use language name as extension for custom languages
        };

        // Generate unique file name
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let file_name = format!("mx_temp_{}.{}", timestamp, file_ext);
        let temp_file = temp_dir.join(&file_name);

        // Write code to temporary file
        fs::write(&temp_file, code)
            .map_err(|e| Error::Execution(format!("Failed to write temp file: {}", e)))?;

        // Execute go run <file>
        let status = Command::new(parts[0])
            .args(&parts[1..])
            .arg(&temp_file)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| Error::Execution(format!("Failed to execute {}: {}", lang, e)))?;

        // Clean up temporary file
        fs::remove_file(&temp_file).ok();

        if !status.success() {
            Err(Error::Execution(format!("{} execution failed", lang)))
        } else {
            Ok(())
        }
    }

    /// Run a specific task by section title
    pub fn run_task<P: AsRef<Path>>(&mut self, markdown_path: P, task_name: &str) -> Result<()> {
        let markdown = self.load_markdown(markdown_path)?;
        let sections = self.extract_sections(&markdown)?;

        let section = self
            .find_section(&sections, task_name)
            .ok_or_else(|| Error::SectionNotFound(task_name.to_string()))?;

        self.execute_section(section)
    }

    /// List all available tasks (sections) in a Markdown file
    pub fn list_tasks<P: AsRef<Path>>(&mut self, markdown_path: P) -> Result<Vec<String>> {
        let markdown = self.load_markdown(markdown_path)?;
        let sections = self.extract_sections(&markdown)?;

        Ok(sections
            .into_iter()
            .map(|s| format!("{}: {}", s.title, s.description.unwrap_or_default()))
            .collect())
    }

    /// List all available task sections in a Markdown file with their details
    pub fn list_task_sections<P: AsRef<Path>>(&mut self, markdown_path: P) -> Result<Vec<Section>> {
        let markdown = self.load_markdown(markdown_path)?;
        self.extract_sections(&markdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = Runner::with_default_config();
        assert_eq!(runner.config.heading_level, 2);
    }

    #[test]
    fn test_extract_sections() {
        let markdown = r#"# Title

## Task 1

```bash
echo "hello"
```

## Task 2

```python
print("world")
```
"#;

        let mut runner = Runner::with_default_config();
        let sections = runner.extract_sections(markdown).unwrap();

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "Task 1");
        assert_eq!(sections[0].codes.len(), 1);
        assert_eq!(sections[0].codes[0].lang, "bash");
    }

    #[test]
    fn test_find_section() {
        let sections = vec![
            Section {
                title: "Task 1".to_string(),
                level: 2,
                ..Default::default()
            },
            Section {
                title: "Task 2".to_string(),
                level: 2,
                ..Default::default()
            },
        ];

        let runner = Runner::with_default_config();
        let found = runner.find_section(&sections, "Task 1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Task 1");

        let not_found = runner.find_section(&sections, "Task 3");
        assert!(not_found.is_none());
    }
}
