use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use serde::Serialize;
use serde_json::Value;
use std::io::{IsTerminal, Write};

/// Output format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Plain,
    Csv,
}

impl OutputFormat {
    pub fn from_flags(json: bool, plain: bool, output: Option<&str>) -> Self {
        if json {
            return OutputFormat::Json;
        }
        if plain {
            return OutputFormat::Plain;
        }
        if let Some(fmt) = output {
            return match fmt.to_lowercase().as_str() {
                "json" => OutputFormat::Json,
                "table" => OutputFormat::Table,
                "plain" | "text" => OutputFormat::Plain,
                "csv" => OutputFormat::Csv,
                _ => OutputFormat::Table,
            };
        }
        // Default: table if TTY, json otherwise
        if std::io::stdout().is_terminal() {
            OutputFormat::Table
        } else {
            OutputFormat::Json
        }
    }
}

/// Renders data to stdout in the requested format
pub struct OutputRenderer;

impl OutputRenderer {
    /// Render a serializable value
    pub fn render<T: Serialize>(
        format: OutputFormat,
        data: &T,
    ) -> Result<(), crate::error::MogError> {
        let value = serde_json::to_value(data)?;
        match format {
            OutputFormat::Json => Self::render_json(&value),
            OutputFormat::Table => Self::render_table(&value),
            OutputFormat::Plain => Self::render_plain(&value),
            OutputFormat::Csv => Self::render_csv(&value),
        }
    }

    /// Render raw JSON value
    pub fn render_value(format: OutputFormat, value: &Value) -> Result<(), crate::error::MogError> {
        match format {
            OutputFormat::Json => Self::render_json(value),
            OutputFormat::Table => Self::render_table(value),
            OutputFormat::Plain => Self::render_plain(value),
            OutputFormat::Csv => Self::render_csv(value),
        }
    }

    fn render_json(value: &Value) -> Result<(), crate::error::MogError> {
        let out = serde_json::to_string_pretty(value)?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "{}", out)?;
        Ok(())
    }

    fn render_table(value: &Value) -> Result<(), crate::error::MogError> {
        match value {
            Value::Array(items) if !items.is_empty() => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic);

                // Extract headers from first object
                if let Some(Value::Object(first)) = items.first() {
                    let headers: Vec<String> = first.keys().cloned().collect();
                    table.set_header(&headers);

                    for item in items {
                        if let Value::Object(obj) = item {
                            let row: Vec<String> = headers
                                .iter()
                                .map(|h| match obj.get(h) {
                                    Some(Value::String(s)) => s.clone(),
                                    Some(Value::Null) => String::new(),
                                    Some(v) => v.to_string(),
                                    None => String::new(),
                                })
                                .collect();
                            table.add_row(row);
                        }
                    }
                }

                println!("{table}");
            }
            Value::Object(_) => {
                // Single object: key-value table
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic);
                table.set_header(vec!["Field", "Value"]);

                if let Value::Object(obj) = value {
                    for (k, v) in obj {
                        let val = match v {
                            Value::String(s) => s.clone(),
                            Value::Null => String::new(),
                            _ => v.to_string(),
                        };
                        table.add_row(vec![k.clone(), val]);
                    }
                }
                println!("{table}");
            }
            _ => {
                // Fallback to JSON
                Self::render_json(value)?;
            }
        }
        Ok(())
    }

    fn render_plain(value: &Value) -> Result<(), crate::error::MogError> {
        match value {
            Value::Array(items) => {
                for item in items {
                    if let Value::Object(obj) = item {
                        let parts: Vec<String> = obj
                            .values()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                Value::Null => String::new(),
                                _ => v.to_string(),
                            })
                            .collect();
                        println!("{}", parts.join("\t"));
                    } else {
                        println!("{}", format_plain_value(item));
                    }
                }
            }
            Value::Object(obj) => {
                for (k, v) in obj {
                    println!("{}: {}", k, format_plain_value(v));
                }
            }
            _ => {
                println!("{}", format_plain_value(value));
            }
        }
        Ok(())
    }

    fn render_csv(value: &Value) -> Result<(), crate::error::MogError> {
        if let Value::Array(items) = value {
            if let Some(Value::Object(first)) = items.first() {
                let headers: Vec<&String> = first.keys().collect();
                println!(
                    "{}",
                    headers
                        .iter()
                        .map(|h| csv_escape(h))
                        .collect::<Vec<_>>()
                        .join(",")
                );

                for item in items {
                    if let Value::Object(obj) = item {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| match obj.get(*h) {
                                Some(Value::String(s)) => csv_escape(s),
                                Some(Value::Null) => String::new(),
                                Some(v) => csv_escape(&v.to_string()),
                                None => String::new(),
                            })
                            .collect();
                        println!("{}", row.join(","));
                    }
                }
            }
        } else {
            Self::render_json(value)?;
        }
        Ok(())
    }

    /// Write a message to stderr (for warnings, hints, progress)
    pub fn stderr(msg: &str) {
        eprintln!("{}", msg);
    }

    /// Write a warning to stderr
    pub fn warn(msg: &str) {
        eprintln!("Warning: {}", msg);
    }
}

fn format_plain_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        _ => v.to_string(),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_flags() {
        assert_eq!(
            OutputFormat::from_flags(true, false, None),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_flags(false, true, None),
            OutputFormat::Plain
        );
        assert_eq!(
            OutputFormat::from_flags(false, false, Some("csv")),
            OutputFormat::Csv
        );
        assert_eq!(
            OutputFormat::from_flags(false, false, Some("json")),
            OutputFormat::Json
        );
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("hello"), "hello");
        assert_eq!(csv_escape("he,llo"), "\"he,llo\"");
        assert_eq!(csv_escape("he\"llo"), "\"he\"\"llo\"");
    }

    #[test]
    fn test_output_format_unknown_string() {
        assert_eq!(
            OutputFormat::from_flags(false, false, Some("xml")),
            OutputFormat::Table
        );
        assert_eq!(
            OutputFormat::from_flags(false, false, Some("TABLE")),
            OutputFormat::Table
        );
        assert_eq!(
            OutputFormat::from_flags(false, false, Some("text")),
            OutputFormat::Plain
        );
    }

    #[test]
    fn test_json_flag_takes_priority() {
        // json flag wins even if plain is also set
        assert_eq!(
            OutputFormat::from_flags(true, true, Some("csv")),
            OutputFormat::Json
        );
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }
}
