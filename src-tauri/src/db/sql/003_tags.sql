CREATE TABLE incident_tags (
    incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (incident_id, tag)
);
CREATE INDEX idx_incident_tags_tag ON incident_tags(tag)