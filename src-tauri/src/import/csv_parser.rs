use std::collections::HashMap;

use crate::error::{AppError, AppResult};

/// Read the first row of a CSV file and return column names.
/// Handles UTF-8 BOM and quoted fields.
pub fn parse_csv_headers(file_path: &str) -> AppResult<Vec<String>> {
    let mut reader = build_reader(file_path)?;
    let headers = reader
        .headers()
        .map_err(|e| AppError::Csv(format!("Failed to read CSV headers: {}", e)))?;

    let cols: Vec<String> = headers.iter().map(|h| strip_bom(h).trim().to_string()).collect();

    if cols.is_empty() {
        return Err(AppError::Csv("CSV file has no columns".into()));
    }

    Ok(cols)
}

/// Parse all data rows into column->value maps.
/// Handles quoted fields, UTF-8 BOM, and skips empty rows.
pub fn parse_csv_rows(file_path: &str) -> AppResult<Vec<HashMap<String, String>>> {
    let mut reader = build_reader(file_path)?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| AppError::Csv(format!("Failed to read CSV headers: {}", e)))?
        .iter()
        .map(|h| strip_bom(h).trim().to_string())
        .collect();

    let mut rows = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| AppError::Csv(format!("Failed to parse CSV row: {}", e)))?;

        // Skip completely empty rows
        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        let mut map = HashMap::new();
        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                map.insert(header.clone(), field.trim().to_string());
            }
        }
        rows.push(map);
    }

    Ok(rows)
}

/// Maximum CSV file size: 10MB
const MAX_CSV_SIZE: u64 = 10 * 1024 * 1024;

fn build_reader(file_path: &str) -> AppResult<csv::Reader<std::fs::File>> {
    // Reject path traversal attempts
    if file_path.contains("..") {
        return Err(AppError::Csv("File path must not contain '..'".into()));
    }

    let file = std::fs::File::open(file_path)
        .map_err(|e| AppError::Csv(format!("Cannot open CSV file: {}", e)))?;

    // Check file size before reading
    let metadata = file.metadata()
        .map_err(|e| AppError::Csv(format!("Cannot read file metadata: {}", e)))?;
    if metadata.len() > MAX_CSV_SIZE {
        return Err(AppError::Csv(format!(
            "CSV file too large ({:.1} MB). Maximum is {} MB.",
            metadata.len() as f64 / (1024.0 * 1024.0),
            MAX_CSV_SIZE / (1024 * 1024)
        )));
    }

    let reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    Ok(reader)
}

/// Strip UTF-8 BOM (byte order mark) from the start of a string
fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("create temp file");
        file.write_all(content.as_bytes()).expect("write csv");
        file.flush().expect("flush");
        file
    }

    #[test]
    fn test_parse_headers() {
        let file = write_csv("Title,Service,Severity\nTest,Web,High\n");
        let headers = parse_csv_headers(file.path().to_str().expect("path")).expect("headers");
        assert_eq!(headers, vec!["Title", "Service", "Severity"]);
    }

    #[test]
    fn test_parse_rows() {
        let file = write_csv("Title,Service\nTest Incident,Web\nAnother,API\n");
        let rows = parse_csv_rows(file.path().to_str().expect("path")).expect("rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("Title").map(|s| s.as_str()), Some("Test Incident"));
        assert_eq!(rows[1].get("Service").map(|s| s.as_str()), Some("API"));
    }

    #[test]
    fn test_skips_empty_rows() {
        let file = write_csv("Title,Service\nTest,Web\n,,\nAnother,API\n");
        let rows = parse_csv_rows(file.path().to_str().expect("path")).expect("rows");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_bom_handling() {
        let content = "\u{feff}Title,Service\nTest,Web\n";
        let file = write_csv(content);
        let headers = parse_csv_headers(file.path().to_str().expect("path")).expect("headers");
        assert_eq!(headers[0], "Title");
    }
}
