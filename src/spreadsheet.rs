use reqwest::blocking;
use csv::ReaderBuilder;

/// Convert a Google Sheets link into a CSV export URL.
/// Works with links ending in /edit, /view, /copy, etc.
pub fn to_csv_url(sheet_url: &str) -> Option<String> {
    let parts: Vec<&str> = sheet_url.split('/').collect();
    // Find the "d" segment and take the next part as the sheet ID
    let id_pos = parts.iter().position(|&p| p == "d")?;
    let sheet_id = parts.get(id_pos + 1)?;
    Some(format!(
        "https://docs.google.com/spreadsheets/d/{}/export?format=csv&gid=0",
        sheet_id
    ))
}

/// Download and parse a CSV into rows of strings.
/// Returns Vec<Vec<String>> where rows[0] is the first row of the sheet.
pub fn load_csv_from_url(url: &str) -> Option<Vec<Vec<String>>> {
    let resp = blocking::get(url).ok()?.text().ok()?;
    let mut rdr = ReaderBuilder::new().from_reader(resp.as_bytes());
    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.ok()?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    Some(rows)
}

pub fn load_csv_from_file(path: &str) -> Option<Vec<Vec<String>>> {
    let mut rdr = csv::Reader::from_path(path).ok()?;
    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.ok()?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    Some(rows)
}