CREATE TABLE report_history (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    quarter_id TEXT REFERENCES quarter_config(id),
    format TEXT NOT NULL DEFAULT 'docx',
    generated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    file_path TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    file_size_bytes INTEGER DEFAULT 0
);
CREATE INDEX idx_report_history_generated_at ON report_history(generated_at)
