-- Migration 023: Enrichment jobs (AI as glue, not truth)

CREATE TABLE IF NOT EXISTS enrichment_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    job_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('queued', 'running', 'succeeded', 'failed')) DEFAULT 'queued',
    input_hash TEXT NOT NULL,
    output_json TEXT NOT NULL DEFAULT '{}',
    model_id TEXT NOT NULL DEFAULT '',
    prompt_version TEXT NOT NULL DEFAULT '',
    error TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_enrichment_jobs_entity ON enrichment_jobs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_enrichment_jobs_created_at ON enrichment_jobs(created_at DESC);

-- Per-incident enrichment materialization (accepted outputs).
CREATE TABLE IF NOT EXISTS incident_enrichments (
    incident_id TEXT PRIMARY KEY NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    executive_summary TEXT NOT NULL DEFAULT '',
    last_job_id TEXT REFERENCES enrichment_jobs(id) ON DELETE SET NULL,
    generated_by TEXT NOT NULL DEFAULT 'manual' CHECK(generated_by IN ('manual', 'ai')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

