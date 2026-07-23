use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Local;
use serde_json::{Value, json};

const LOG_FILE_PREFIX: &str = "stellune-";
const LOG_FILE_SUFFIX: &str = ".jsonl";
const LOG_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const REDACTED: &str = "[REDACTED]";

#[derive(Clone)]
pub struct AppLogger {
    log_dir: Arc<Option<PathBuf>>,
    write_lock: Arc<Mutex<()>>,
}

impl AppLogger {
    pub fn new(app_data_dir: &Path) -> Self {
        let log_dir = app_data_dir.join("logs");
        let log_dir = fs::create_dir_all(&log_dir)
            .map(|_| {
                cleanup_old_logs(&log_dir);
                log_dir
            })
            .ok();
        Self {
            log_dir: Arc::new(log_dir),
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn info(&self, event: &str, correlation_id: &str, payload: Value) {
        self.write("info", event, correlation_id, payload);
    }

    pub fn error(&self, event: &str, correlation_id: &str, payload: Value) {
        self.write("error", event, correlation_id, payload);
    }

    pub fn external_request(
        &self,
        correlation_id: &str,
        provider: &str,
        endpoint: &str,
        body: Value,
    ) {
        self.info(
            "external.request",
            correlation_id,
            json!({
                "provider": provider,
                "endpoint": endpoint,
                "body": body,
            }),
        );
    }

    pub fn external_response(
        &self,
        correlation_id: &str,
        provider: &str,
        status: u16,
        body: Value,
    ) {
        self.info(
            "external.response",
            correlation_id,
            json!({
                "provider": provider,
                "status": status,
                "body": body,
            }),
        );
    }

    pub fn generation_input(&self, correlation_id: &str, generator: &str, input: Value) {
        self.info(
            "generation.input",
            correlation_id,
            json!({
                "generator": generator,
                "input": input,
            }),
        );
    }

    pub fn generation_output(&self, correlation_id: &str, generator: &str, output: Value) {
        self.info(
            "generation.output",
            correlation_id,
            json!({
                "generator": generator,
                "output": output,
            }),
        );
    }

    fn write(&self, level: &str, event: &str, correlation_id: &str, payload: Value) {
        let Ok(_guard) = self.write_lock.lock() else {
            return;
        };
        let line = json!({
            "timestamp": Local::now().to_rfc3339(),
            "level": level,
            "event": event,
            "correlationId": correlation_id,
            "payload": redact(payload),
        });
        let Some(path) = self.current_log_path() else {
            return;
        };
        let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
            return;
        };
        if serde_json::to_writer(&mut file, &line).is_ok() {
            let _ = file.write_all(b"\n");
        }
    }

    fn current_log_path(&self) -> Option<PathBuf> {
        self.log_dir.as_ref().as_ref().map(|log_dir| {
            log_dir.join(format!(
                "{LOG_FILE_PREFIX}{}{LOG_FILE_SUFFIX}",
                Local::now().format("%Y-%m-%d")
            ))
        })
    }
}

fn cleanup_old_logs(log_dir: &Path) {
    let Ok(entries) = fs::read_dir(log_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(LOG_FILE_PREFIX) || !name.ends_with(LOG_FILE_SUFFIX) {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if modified.elapsed().is_ok_and(|age| age > LOG_RETENTION) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn redact(value: Value) -> Value {
    match value {
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| {
                    if is_sensitive_key(&key) {
                        (key, Value::String(REDACTED.into()))
                    } else {
                        (key, redact(value))
                    }
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact).collect()),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    let contains_secret = [
        "apikey",
        "token",
        "secret",
        "authorization",
        "password",
        "credential",
    ]
    .iter()
    .any(|candidate| normalized.contains(candidate));
    let is_private_profile_field = [
        "lat",
        "lon",
        "latitude",
        "tzone",
        "timezone",
        "birthday",
        "birthdate",
        "birthcity",
        "nickname",
    ]
    .contains(&normalized.as_str());
    contains_secret || is_private_profile_field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_jsonl_and_redacts_nested_secrets() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let logger = AppLogger::new(directory.path());
        logger.external_request(
            "request-1",
            "deepseek",
            "https://example.test",
            json!({
                "prompt": "hello",
                "api_key": "secret-value",
                "lat": 31.2304,
                "lon": 121.4737,
                "headers": {
                    "Authorization": "Bearer private-token"
                }
            }),
        );

        let content = fs::read_to_string(logger.current_log_path().expect("enabled log"))
            .expect("log content");
        assert!(content.contains("\"event\":\"external.request\""));
        assert!(content.contains("\"correlationId\":\"request-1\""));
        assert!(content.contains("\"prompt\":\"hello\""));
        assert!(content.contains(REDACTED));
        assert!(!content.contains("secret-value"));
        assert!(!content.contains("private-token"));
        assert!(!content.contains("31.2304"));
        assert!(!content.contains("121.4737"));
    }

    #[test]
    fn directory_failure_disables_logging_without_failing_the_caller() {
        let directory = tempfile::tempdir().expect("temporary directory");
        fs::write(directory.path().join("logs"), "occupied").expect("blocking file");

        let logger = AppLogger::new(directory.path());
        logger.info("test.event", "request-1", json!({"ok": true}));

        assert!(logger.current_log_path().is_none());
    }
}
