use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub file_name: String,
    pub name: String,
    pub columns: Vec<Column>,
}

/// Maximum size a workspace file may have to be read into memory.
const MAX_FILE_SIZE: u64 = 32 * 1024 * 1024;

/// First line of a v3 table file. Values are stored escaped, so any text
/// (including `|`, `\` and even newlines) can live in a cell without corrupting
/// the row layout. Table files WITHOUT this header keep the legacy plain
/// `|`-separated format and are only ever read.
pub const FORMAT_HEADER: &str = "# fakesql format 3";

/// How long an advisory lock file may be idle before being considered stale.
const LOCK_STALE: Duration = Duration::from_secs(15);

/// How long acquire_lock is willing to wait for a live lock to be released.
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);

const RESERVED_TABLE_NAMES: &[&str] = &[
    "manager",
    "workspace",
    "con", "prn", "aux", "nul",
    "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
    "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

pub fn validate_table_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Le nom de table est vide".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(format!(
            "Nom de table invalide '{}' : utilisez lettres, chiffres, '_' ou '-'",
            name
        ));
    }
    if RESERVED_TABLE_NAMES.contains(&name.to_ascii_lowercase().as_str()) {
        return Err(format!(
            "Nom de table réservé '{}' : choisissez un autre nom",
            name
        ));
    }
    Ok(())
}

pub fn manager_path(root: &Path) -> PathBuf {
    root.join("manager.txt")
}

pub fn table_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    validate_table_name(name)?;
    Ok(root.join(format!("{}.txt", name)))
}

fn strip_row_suffix(value: &str) -> &str {
    if let Some((head, tail)) = value.rsplit_once('*') {
        if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
            return head;
        }
    }
    value
}

/// Encodes a cell value for the v3 table format, escaping the characters that
/// carry structure (`\`, `|`, and newlines).
fn encode_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '|' => out.push_str("\\|"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Splits a v3 row line on unescaped `|` and decodes each cell, reversing
/// `encode_value`.
fn decode_row(line: &str) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => current.push('\\'),
                Some('|') => current.push('|'),
                Some('n') => current.push('\n'),
                Some('r') => current.push('\r'),
                // Not produced by the encoder (hand-edited file): keep it as-is.
                Some(other) => {
                    current.push('\\');
                    current.push(other);
                }
                None => current.push('\\'),
            }
        } else if c == '|' {
            values.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }
    values.push(current);
    values
}

/// Parses table content. One row per physical line; values are decoded in the
/// v3 format. The header itself is not a row.
fn parse_table(content: &str) -> Vec<Vec<String>> {
    let mut lines = content.lines();
    let is_v3 = lines
        .next()
        .map(|line| line.trim() == FORMAT_HEADER)
        .unwrap_or(false);

    if !is_v3 {
        // Legacy files: plain `|` split, no decoding, no escapes.
        return content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().split('|').map(|v| v.to_string()).collect())
            .collect();
    }

    lines
        .filter(|line| !line.trim().is_empty())
        .map(decode_row)
        .collect()
}

/// Reads a text file while guarding memory: refuses files above the size cap
/// and strips a UTF-8 BOM if one was added by an external editor.
pub fn read_text(path: &Path) -> Result<Option<String>, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!(
            "Fichier trop volumineux : '{}' dépasse 32 Mo",
            path.display()
        ));
    }
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    Ok(Some(
        content
            .strip_prefix('\u{feff}')
            .unwrap_or(&content)
            .to_string(),
    ))
}

/// Writes a file through a temporary sibling and a rename, so an interrupted
/// write can never leave a truncated file behind.
pub fn write_text_atomic(path: &Path, content: &str) -> Result<(), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let temporary = path.with_file_name(format!(".{}.tmp", file_name));

    if let Err(error) = fs::write(&temporary, content) {
        return Err(error.to_string());
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(())
}

/// Advisory lock acquired around every mutating workspace operation. Another
/// FakeSQL Studio instance working on the same folder is refused until the
/// lock becomes stale.
pub struct WorkspaceLock {
    path: PathBuf,
}

impl Drop for WorkspaceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn acquire_lock(root: &Path) -> Result<WorkspaceLock, String> {
    let path = root.join("workspace.lock");
    let deadline = std::time::Instant::now() + LOCK_TIMEOUT;

    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                let _ = std::io::Write::write_all(
                    &mut file,
                    format!("{}\n", std::process::id()).as_bytes(),
                );
                return Ok(WorkspaceLock { path });
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                let age = fs::metadata(&path)
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| {
                        modified
                            .elapsed()
                            .map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))
                            .ok()
                    })
                    .unwrap_or_else(|| Duration::from_secs(0));
                if age > LOCK_STALE {
                    let _ = fs::remove_file(&path);
                    continue;
                }
                if std::time::Instant::now() >= deadline {
                    return Err(
                        "Le workspace est verrouillé par une autre instance de FakeSQL Studio"
                            .to_string(),
                    );
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

/// Loads the catalog and reports every problem found along the way
/// (malformed lines, invalid table names, unexpected column formats).
pub fn load_manager_report(
    root: &Path,
) -> Result<(Vec<TableSchema>, Vec<String>), String> {
    let Some(content) = read_text(&manager_path(root))? else {
        return Ok((Vec::new(), Vec::new()));
    };

    let mut tables = Vec::new();
    let mut warnings = Vec::new();
    for (index, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((file_name, schema_part)) = line.split_once(" > ") else {
            warnings.push(format!(
                "Ligne {} de manager.txt ignorée : format invalide",
                index + 1
            ));
            continue;
        };
        let file_name = file_name.trim().to_string();
        if file_name.is_empty() {
            warnings.push(format!(
                "Ligne {} de manager.txt ignorée : format invalide",
                index + 1
            ));
            continue;
        }
        let name = file_name
            .strip_suffix(".txt")
            .unwrap_or(&file_name)
            .to_string();
        if validate_table_name(&name).is_err() {
            warnings.push(format!(
                "Ligne {} de manager.txt ignorée : nom de table invalide '{}'",
                index + 1,
                file_name
            ));
            continue;
        }

        let mut columns = Vec::new();
        let mut bad_columns = Vec::new();
        for part in strip_row_suffix(schema_part.trim()).split('|') {
            if part.trim().is_empty() {
                continue;
            }
            let Some((column_name, kind)) = part.split_once(':') else {
                bad_columns.push(part.trim().to_string());
                continue;
            };
            let column_name = column_name.trim();
            let kind = kind.trim();
            if !column_name.is_empty() && !kind.is_empty() {
                columns.push(Column {
                    name: column_name.to_string(),
                    kind: kind.to_string(),
                });
            } else {
                bad_columns.push(part.trim().to_string());
            }
        }
        for bad_column in bad_columns {
            warnings.push(format!(
                "Ligne {} de manager.txt : colonne au format inattendu '{}'",
                index + 1,
                bad_column
            ));
        }
        if columns.is_empty() {
            warnings.push(format!(
                "Ligne {} de manager.txt : schéma vide pour '{}'",
                index + 1,
                file_name
            ));
            continue;
        }

        tables.push(TableSchema {
            file_name,
            name,
            columns,
        });
    }

    Ok((tables, warnings))
}

pub fn load_manager(root: &Path) -> Result<Vec<TableSchema>, String> {
    Ok(load_manager_report(root)?.0)
}

pub fn count_rows(root: &Path, name: &str) -> Result<usize, String> {
    let path = table_path(root, name)?;
    match read_text(&path)? {
        Some(content) => Ok(parse_table(&content).len()),
        None => Ok(0),
    }
}

/// Writes the catalog. The old `*rowcount` suffix is deliberately not
/// produced anymore: the count is derived from the table file itself, so the
/// catalog is only touched when the list of tables changes. Old catalogs
/// carrying a `*N` suffix are still read without issue via `strip_row_suffix`.
pub fn save_manager(root: &Path, tables: &[TableSchema]) -> Result<(), String> {
    let mut output = String::new();
    for table in tables {
        let schema = table
            .columns
            .iter()
            .map(|column| format!("{}:{}", column.name, column.kind))
            .collect::<Vec<_>>()
            .join("|");
        output.push_str(&format!("{} > {}\n", table.file_name, schema));
    }
    write_text_atomic(&manager_path(root), &output)
}

pub fn find_table(root: &Path, name: &str) -> Result<Option<TableSchema>, String> {
    Ok(load_manager(root)?
        .into_iter()
        .find(|table| table.name == name))
}

pub fn load_rows(root: &Path, name: &str) -> Result<Vec<Vec<String>>, String> {
    let path = table_path(root, name)?;
    let Some(content) = read_text(&path)? else {
        return Ok(Vec::new());
    };

    Ok(parse_table(&content))
}

/// Writes the data file in the v3 format: a header line, then one row per
/// line with escaped cell values. Old files without a header remain readable
/// via `parse_table`.
pub fn save_rows(root: &Path, name: &str, rows: &[Vec<String>]) -> Result<(), String> {
    let path = table_path(root, name)?;
    let mut output = String::new();
    output.push_str(FORMAT_HEADER);
    output.push('\n');
    for row in rows {
        let encoded: Vec<String> = row.iter().map(|value| encode_value(value)).collect();
        output.push_str(&encoded.join("|"));
        output.push('\n');
    }
    let _guard = acquire_lock(root)?;
    write_text_atomic(&path, &output)
}

pub fn create_table(root: &Path, name: &str, columns: &[Column]) -> Result<TableSchema, String> {
    validate_table_name(name)?;
    let mut tables = load_manager(root)?;
    let _guard = acquire_lock(root)?;

    if tables.iter().any(|table| table.name == name) {
        return Err(format!("La table '{}' existe déjà", name));
    }
    let path = table_path(root, name)?;
    if path.exists() {
        return Err(format!("'{}.txt' existe déjà", name));
    }

    write_text_atomic(&path, &format!("{}\n", FORMAT_HEADER))?;
    let schema = TableSchema {
        file_name: format!("{}.txt", name),
        name: name.to_string(),
        columns: columns.to_vec(),
    };
    tables.push(schema.clone());
    save_manager(root, &tables)?;
    Ok(schema)
}

pub fn delete_table(root: &Path, name: &str) -> Result<(), String> {
    let mut tables = load_manager(root)?;
    let Some(index) = tables.iter().position(|table| table.name == name) else {
        return Err(format!("La table '{}' n'existe pas", name));
    };
    let _guard = acquire_lock(root)?;
    tables.remove(index);

    let path = table_path(root, name)?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    save_manager(root, &tables)
}

pub fn rename_table(root: &Path, from: &str, to: &str) -> Result<TableSchema, String> {
    validate_table_name(to)?;
    let mut tables = find_table(root, from)?.ok_or_else(|| format!("La table '{}' n'existe pas", from))?;
    let mut all_tables = load_manager(root)?;

    if all_tables.iter().any(|table| table.name == to) {
        return Err(format!("La table '{}' existe déjà", to));
    }
    let old_path = table_path(root, from)?;
    let new_path = table_path(root, to)?;
    let _guard = acquire_lock(root)?;

    if !old_path.exists() {
        return Err(format!(
            "La table '{}' n'a pas de fichier de données",
            from
        ));
    }
    if new_path.exists() {
        return Err(format!("'{}.txt' existe déjà", to));
    }

    fs::rename(&old_path, &new_path).map_err(|error| error.to_string())?;
    all_tables.retain(|table| table.name != from);
    tables.name = to.to_string();
    tables.file_name = format!("{}.txt", to);
    all_tables.push(tables.clone());
    save_manager(root, &all_tables)?;
    Ok(tables)
}

/// Reports workspace-level inconsistencies: malformed catalog lines and
/// orphan table files. Surfaced to the UI's console instead of being
/// silently swallowed.
pub fn workspace_issues(root: &Path) -> Result<Vec<String>, String> {
    let (tables, warnings) = load_manager_report(root)?;
    let mut issues = warnings;

    let entries = fs::read_dir(root).map_err(|error| error.to_string())?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".txt") || name.starts_with('.') || name == "manager.txt" {
            continue;
        }
        let Some(stem) = name.strip_suffix(".txt") else {
            continue;
        };
        if validate_table_name(stem).is_err() {
            continue;
        }
        if !tables.iter().any(|table| table.file_name == name) {
            issues.push(format!(
                "Fichier orphelin : '{}' n'a pas d'entrée dans manager.txt",
                name
            ));
        }
    }

    for table in tables {
        if !table_path(root, &table.name)?.exists() {
            issues.push(format!(
                "La table '{}' est référencée dans manager.txt mais son fichier est absent",
                table.name
            ));
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_structural_characters_losslessly() {
        for value in ["", "plain", "a|b", "back\\slash", "ligne\nsuivante", "|", "\\", "a > b"] {
            let decoded = decode_row(&encode_value(value));
            assert_eq!(decoded, vec![value.to_string()], "valeur : {value:?}");
        }
    }

    #[test]
    fn splits_encoded_rows_on_unescaped_pipes() {
        assert_eq!(decode_row("1|a\\|b|c"), vec!["1", "a|b", "c"]);
    }

    #[test]
    fn parses_v3_and_legacy_files() {
        let v3 = format!("{}\n1|a\\|b\n2|c\n", FORMAT_HEADER);
        assert_eq!(
            parse_table(&v3),
            vec![
                vec!["1".to_string(), "a|b".to_string()],
                vec!["2".to_string(), "c".to_string()],
            ]
        );

        // Legacy files stay readable with the plain split.
        let legacy = "1|youssef|0000\n3|aa|1retrh\n";
        assert_eq!(
            parse_table(legacy),
            vec![
                vec!["1".to_string(), "youssef".to_string(), "0000".to_string()],
                vec!["3".to_string(), "aa".to_string(), "1retrh".to_string()],
            ]
        );
    }

    #[test]
    fn counts_only_v3_header_removed() {
        let v3 = format!("{}\n1|a\n\n2|b\n", FORMAT_HEADER);
        assert_eq!(parse_table(&v3).len(), 2);
    }
}
