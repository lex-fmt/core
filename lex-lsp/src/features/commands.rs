use serde_json::Value;
use tower_lsp::jsonrpc::{Error, Result};

pub const COMMAND_ECHO: &str = "lex.echo";

pub fn execute_command(command: &str, arguments: &[Value]) -> Result<Option<Value>> {
    match command {
        COMMAND_ECHO => {
            let msg = arguments
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("default echo");
            Ok(Some(Value::String(format!("Echo: {}", msg))))
        }
        _ => Err(Error::invalid_request()),
    }
}
