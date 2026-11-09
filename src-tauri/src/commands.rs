use serde::{Deserialize, Serialize};
use tauri::State;

use crate::engine::{
    self,
    executor::QueryResult,
    storage::{self, Column},
    ValidationResult,
};
use crate::state::WorkspaceState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<Column>,
    pub row_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumn {
    pub name: String,
    pub kind: String,
}

#[tauri::command]
pub fn open_workspace(path: String, state: State<'_, WorkspaceState>) -> Result<String, String> {
    let root = std::fs::canonicalize(&path)
        .map_err(|error| format!("Dossier invalide '{}': {}", path, error))?;
    if !root.is_dir() {
        return Err(format!("'{}' n'est pas un dossier", path));
    }
    state.set(root.clone())?;
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_workspace(state: State<'_, WorkspaceState>) -> Option<String> {
    state
        .try_root()
        .map(|root| root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_tables(state: State<'_, WorkspaceState>) -> Result<Vec<TableInfo>, String> {
    let root = state.root()?;
    let tables = storage::load_manager(&root)?;
    let mut result = Vec::with_capacity(tables.len());
    for table in tables {
        let row_count = storage::count_rows(&root, &table.name)?;
        result.push(TableInfo {
            name: table.name,
            columns: table.columns,
            row_count,
        });
    }
    Ok(result)
}

/// Non-blocking warnings about the workspace (malformed catalog lines,
/// orphan table files), shown in the UI console on refresh.
#[tauri::command]
pub fn workspace_issues(state: State<'_, WorkspaceState>) -> Result<Vec<String>, String> {
    let root = state.root()?;
    storage::workspace_issues(&root)
}

/// Runs an editor document as a sequence of statements, stopping at the first
/// failure. Each result carries per-statement timing and outcome.
#[tauri::command]
pub fn run_query(query: String, state: State<'_, WorkspaceState>) -> Result<Vec<QueryResult>, String> {
    let root = state.root()?;
    Ok(engine::executor::run_script(&root, &query))
}

/// Validates every statement of a document at once.
#[tauri::command]
pub fn validate_query(query: String) -> ValidationResult {
    engine::validate_query(&query)
}

/// Writes CSV content produced by the frontend to a user-chosen destination,
/// through the same atomic write used for workspace data.
#[tauri::command]
pub fn export_csv(path: String, content: String) -> Result<String, String> {
    let file = std::path::Path::new(&path);
    if let Some(parent) = file.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| error.to_string())?;
        }
    }
    storage::write_text_atomic(file, &content)
        .map_err(|error| format!("Export impossible : {}", error))?;
    Ok(format!("Résultats exportés vers '{}'", path))
}

#[tauri::command]
pub fn create_table(
    name: String,
    columns: Vec<CreateColumn>,
    state: State<'_, WorkspaceState>,
) -> Result<TableInfo, String> {
    let root = state.root()?;
    let name = name.trim().trim_start_matches('$').to_string();
    storage::validate_table_name(&name)?;

    if columns.is_empty() {
        return Err("Au moins une colonne est requise".to_string());
    }

    let mut parsed = Vec::with_capacity(columns.len());
    for column in columns {
        let column_name = column.name.trim().trim_start_matches('$').to_string();
        if column_name.is_empty() {
            return Err("Le nom de colonne est vide".to_string());
        }
        if !column_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(format!("Nom de colonne invalide '{}'", column_name));
        }
        if parsed
            .iter()
            .any(|existing: &Column| existing.name == column_name)
        {
            return Err(format!("La colonne '{}' est donnée deux fois", column_name));
        }
        let kind = column.kind.trim().to_lowercase();
        if kind != "entier" && kind != "chaine" {
            return Err(format!("Type invalide pour '{}': entier ou chaine", column_name));
        }
        parsed.push(Column {
            name: column_name,
            kind,
        });
    }

    let schema = storage::create_table(&root, &name, &parsed)?;
    Ok(TableInfo {
        name: schema.name,
        columns: schema.columns,
        row_count: 0,
    })
}

#[tauri::command]
pub fn delete_table(name: String, state: State<'_, WorkspaceState>) -> Result<String, String> {
    let root = state.root()?;
    storage::delete_table(&root, &name)?;
    Ok(format!("Table '{}' supprimée", name))
}
