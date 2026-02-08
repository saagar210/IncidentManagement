use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single mapped incident row ready for import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedIncident {
    pub title: String,
    pub service_name: String,
    pub severity: String,
    pub impact: String,
    pub status: String,
    pub started_at: String,
    pub detected_at: String,
    pub responded_at: Option<String>,
    pub resolved_at: Option<String>,
    pub root_cause: String,
    pub resolution: String,
    pub tickets_submitted: i64,
    pub affected_users: i64,
    pub is_recurring: bool,
    pub lessons_learned: String,
    pub external_ref: String,
    pub notes: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Column mapping: CSV column name -> incident field name.
/// Also holds default values for unmapped fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub mappings: HashMap<String, String>,
    pub default_values: HashMap<String, String>,
}

/// All incident fields that can be mapped to.
#[allow(dead_code)]
pub const INCIDENT_FIELDS: &[&str] = &[
    "title",
    "service",
    "severity",
    "impact",
    "status",
    "started_at",
    "detected_at",
    "responded_at",
    "resolved_at",
    "root_cause",
    "resolution",
    "tickets_submitted",
    "affected_users",
    "is_recurring",
    "lessons_learned",
    "external_ref",
    "notes",
];

/// Required fields that must be present (either mapped or have default values).
const REQUIRED_FIELDS: &[&str] = &[
    "title",
    "service",
    "severity",
    "impact",
    "status",
    "started_at",
    "detected_at",
];

const VALID_SEVERITIES: &[&str] = &["Critical", "High", "Medium", "Low"];
const VALID_IMPACTS: &[&str] = &["Critical", "High", "Medium", "Low"];
const VALID_STATUSES: &[&str] = &["Active", "Monitoring", "Resolved", "Post-Mortem"];

/// Apply the column mapping to parsed CSV rows and validate each row.
pub fn apply_mapping(
    rows: &[HashMap<String, String>],
    mapping: &ColumnMapping,
) -> Vec<MappedIncident> {
    // Build reverse mapping: incident_field -> csv_column
    let reverse: HashMap<&str, &str> = mapping
        .mappings
        .iter()
        .map(|(csv_col, field)| (field.as_str(), csv_col.as_str()))
        .collect();

    rows.iter()
        .enumerate()
        .map(|(idx, row)| map_single_row(idx, row, &reverse, &mapping.default_values))
        .collect()
}

/// Try to auto-detect column name matches for common patterns.
#[allow(dead_code)]
pub fn auto_detect_mappings(csv_columns: &[String]) -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    for col in csv_columns {
        let lower = col.to_lowercase().replace([' ', '_', '-'], "");
        let field = match lower.as_str() {
            "title" | "incidenttitle" | "name" | "incidentname" | "summary" => Some("title"),
            "service" | "servicename" | "serviceid" | "system" | "application" => Some("service"),
            "severity" | "sev" | "severitylevel" => Some("severity"),
            "impact" | "impactlevel" => Some("impact"),
            "status" | "state" | "incidentstatus" => Some("status"),
            "startedat" | "startdate" | "starttime" | "start" | "incidentstart" | "began" => {
                Some("started_at")
            }
            "detectedat" | "detectdate" | "detected" | "detectiontime" | "discoveredat" => {
                Some("detected_at")
            }
            "respondedat" | "responsetime" | "responded" | "acknowledged" | "acknowledgedat" => {
                Some("responded_at")
            }
            "resolvedat" | "resolutiontime" | "resolved" | "enddate" | "endtime" | "end" => {
                Some("resolved_at")
            }
            "rootcause" | "cause" => Some("root_cause"),
            "resolution" | "fix" | "remediation" => Some("resolution"),
            "ticketssubmitted" | "tickets" | "ticketcount" => Some("tickets_submitted"),
            "affectedusers" | "users" | "usercount" | "usersaffected" => Some("affected_users"),
            "isrecurring" | "recurring" | "recurrence" => Some("is_recurring"),
            "lessonslearned" | "lessons" | "takeaways" => Some("lessons_learned"),
            "externalref" | "externalreference" | "ticketid" | "jira" | "ref" | "reference" => {
                Some("external_ref")
            }
            "notes" | "comments" | "description" | "details" => Some("notes"),
            _ => None,
        };

        if let Some(f) = field {
            mappings.insert(col.clone(), f.to_string());
        }
    }

    mappings
}

fn map_single_row(
    row_idx: usize,
    row: &HashMap<String, String>,
    reverse: &HashMap<&str, &str>,
    defaults: &HashMap<String, String>,
) -> MappedIncident {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let get_val = |field: &str| -> String {
        if let Some(csv_col) = reverse.get(field) {
            if let Some(val) = row.get(*csv_col) {
                if !val.is_empty() {
                    return sanitize_csv_field(val);
                }
            }
        }
        defaults.get(field).cloned().unwrap_or_default()
    };

    let title = get_val("title");
    if title.is_empty() {
        errors.push(format!("Row {}: Title is required", row_idx + 1));
    }

    let service_name = get_val("service");
    if service_name.is_empty() {
        errors.push(format!("Row {}: Service is required", row_idx + 1));
    }

    let severity = get_val("severity");
    if severity.is_empty() {
        errors.push(format!("Row {}: Severity is required", row_idx + 1));
    } else if !VALID_SEVERITIES.contains(&severity.as_str()) {
        // Try case-insensitive match
        let matched = VALID_SEVERITIES
            .iter()
            .find(|s| s.eq_ignore_ascii_case(&severity));
        if matched.is_none() {
            warnings.push(format!(
                "Row {}: Unknown severity '{}', must be one of: {}",
                row_idx + 1,
                severity,
                VALID_SEVERITIES.join(", ")
            ));
        }
    }

    let impact = get_val("impact");
    if impact.is_empty() {
        errors.push(format!("Row {}: Impact is required", row_idx + 1));
    } else if !VALID_IMPACTS.contains(&impact.as_str()) {
        let matched = VALID_IMPACTS
            .iter()
            .find(|s| s.eq_ignore_ascii_case(&impact));
        if matched.is_none() {
            warnings.push(format!(
                "Row {}: Unknown impact '{}', must be one of: {}",
                row_idx + 1,
                impact,
                VALID_IMPACTS.join(", ")
            ));
        }
    }

    let status = get_val("status");
    if status.is_empty() {
        errors.push(format!("Row {}: Status is required", row_idx + 1));
    } else if !VALID_STATUSES.contains(&status.as_str()) {
        let matched = VALID_STATUSES
            .iter()
            .find(|s| s.eq_ignore_ascii_case(&status));
        if matched.is_none() {
            warnings.push(format!(
                "Row {}: Unknown status '{}', must be one of: {}",
                row_idx + 1,
                status,
                VALID_STATUSES.join(", ")
            ));
        }
    }

    let started_at = get_val("started_at");
    if started_at.is_empty() {
        errors.push(format!("Row {}: Started at is required", row_idx + 1));
    }

    let detected_at = get_val("detected_at");
    if detected_at.is_empty() {
        errors.push(format!("Row {}: Detected at is required", row_idx + 1));
    }

    // Check for unmapped required fields
    for &field in REQUIRED_FIELDS {
        let val = get_val(field);
        if val.is_empty() && !errors.iter().any(|e| e.contains(field)) {
            errors.push(format!("Row {}: Required field '{}' is empty", row_idx + 1, field));
        }
    }

    let responded_at_val = get_val("responded_at");
    let resolved_at_val = get_val("resolved_at");

    let tickets_submitted = get_val("tickets_submitted")
        .parse::<i64>()
        .unwrap_or(0);
    let affected_users = get_val("affected_users")
        .parse::<i64>()
        .unwrap_or(0);
    let is_recurring = matches!(
        get_val("is_recurring").to_lowercase().as_str(),
        "true" | "yes" | "1" | "y"
    );

    MappedIncident {
        title,
        service_name,
        severity: normalize_enum_value(&severity, VALID_SEVERITIES),
        impact: normalize_enum_value(&impact, VALID_IMPACTS),
        status: normalize_enum_value(&status, VALID_STATUSES),
        started_at,
        detected_at,
        responded_at: if responded_at_val.is_empty() {
            None
        } else {
            Some(responded_at_val)
        },
        resolved_at: if resolved_at_val.is_empty() {
            None
        } else {
            Some(resolved_at_val)
        },
        root_cause: get_val("root_cause"),
        resolution: get_val("resolution"),
        tickets_submitted,
        affected_users,
        is_recurring,
        lessons_learned: get_val("lessons_learned"),
        external_ref: get_val("external_ref"),
        notes: get_val("notes"),
        warnings,
        errors,
    }
}

/// Try to normalize an enum value to its canonical form (case-insensitive match).
fn normalize_enum_value(value: &str, valid: &[&str]) -> String {
    if valid.contains(&value) {
        return value.to_string();
    }
    valid
        .iter()
        .find(|v| v.eq_ignore_ascii_case(value))
        .map(|v| v.to_string())
        .unwrap_or_else(|| value.to_string())
}

/// Sanitize a CSV field to prevent formula injection in downstream tools.
/// Prefixes dangerous leading characters with a single quote.
/// Covers OWASP recommendations: =, +, -, @, \t, \r, | (pipe/cmd), { (SLK format).
fn sanitize_csv_field(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }
    match trimmed.as_bytes()[0] {
        b'=' | b'+' | b'@' | b'\t' | b'\r' | b'|' | b'{' => format!("'{}", trimmed),
        b'-' if trimmed.len() > 1 && !trimmed[1..].starts_with(|c: char| c.is_ascii_digit()) => {
            format!("'{}", trimmed)
        }
        _ => trimmed.to_string(),
    }
}
