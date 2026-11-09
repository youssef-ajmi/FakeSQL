use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::storage::{self, TableSchema};
use super::token::Token;
use super::{lexer, validator};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub command: String,
    pub success: bool,
    pub message: Option<String>,
    pub columns: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<String>>>,
    pub row_count: Option<usize>,
    pub affected: Option<usize>,
    pub elapsed_ms: u64,
}

impl QueryResult {
    fn failure(command: &str, message: impl Into<String>, started: Instant) -> Self {
        Self {
            command: command.to_string(),
            success: false,
            message: Some(message.into()),
            columns: None,
            rows: None,
            row_count: None,
            affected: None,
            elapsed_ms: started.elapsed().as_millis() as u64,
        }
    }

    fn success(command: &str, message: impl Into<String>) -> Self {
        Self {
            command: command.to_string(),
            success: true,
            message: Some(message.into()),
            columns: None,
            rows: None,
            row_count: None,
            affected: None,
            elapsed_ms: 0,
        }
    }
}

fn command_of(tokens: &[Token]) -> String {
    tokens
        .first()
        .map(|token| token.text.trim_matches(&['[', ']'][..]).to_string())
        .unwrap_or_default()
}

fn is_quoted(token: &Token) -> bool {
    let bytes = token.text.as_bytes();
    bytes.len() >= 4 && bytes[1] == b'\'' && bytes[bytes.len() - 2] == b'\''
}

fn is_digits(token: &Token) -> bool {
    let inner = &token.text[1..token.text.len() - 1];
    !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit())
}

fn table_name(token: &Token) -> Result<String, String> {
    if token.text.len() < 4 {
        return Err("Nom de table invalide".to_string());
    }
    let name = &token.text[2..token.text.len() - 2];
    storage::validate_table_name(name)?;
    Ok(name.to_string())
}

fn var_name(token: &Token) -> String {
    token.text[2..token.text.len() - 1].to_string()
}

fn token_value(token: &Token) -> String {
    if is_quoted(token) {
        // '' is an escaped apostrophe: it decodes to a single '.
        token.text[2..token.text.len() - 2].replace("''", "'")
    } else {
        token.text[1..token.text.len() - 1].to_string()
    }
}

fn strip_brackets(token: &Token) -> &str {
    &token.text[1..token.text.len() - 1]
}

/// Bare decimal literal `[3.5]` etc.
fn is_number(token: &Token) -> bool {
    let inner = &token.text[1..token.text.len() - 1];
    let mut parts = inner.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(head), None, None) => {
            !head.is_empty() && head.chars().all(|c| c.is_ascii_digit())
        }
        (Some(head), Some(tail), None) => {
            !head.is_empty()
                && !tail.is_empty()
                && head.chars().all(|c| c.is_ascii_digit())
                && tail.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

/// Bare boolean literal `vrai` / `faux` (the lexer uppercases bare words).
fn is_bool(token: &Token) -> bool {
    matches!(&token.text[1..token.text.len() - 1], "VRAI" | "FAUX")
}

/// Strict data typing: a bare digit token is an `entier`, a bare decimal is a
/// `reel`, `vrai`/`faux` is `booleen`, and quoted tokens are `chaine`
/// (a `caractere` holds exactly one character). A quoted number is no longer
/// accepted where a number is expected.
fn token_matches_strict(token: &Token, kind: &str) -> bool {
    match kind {
        "entier" => !is_quoted(token) && is_digits(token),
        "reel" => !is_quoted(token) && is_number(token),
        "booleen" => is_bool(token),
        "caractere" => is_quoted(token) && {
            let value = token_value(token);
            value.chars().count() == 1
        },
        "chaine" => is_quoted(token),
        _ => false,
    }
}

fn type_error(column: &str, kind: &str) -> String {
    format!("'${}' doit être de type {}", column, kind)
}

/// Value written to storage: booleans are stored lowercase (`vrai`/`faux`)
/// so the files keep a canonical form; other kinds keep their source text.
fn stored_value(kind: &str, token: &Token) -> String {
    let value = token_value(token);
    if kind == "booleen" {
        return value.to_lowercase();
    }
    value
}

/// Equality between a stored cell and a value. Integer columns are compared
/// numerically ("01" equals "1"), real columns numerically as f64, boolean
/// columns case-insensitively; anything that does not parse falls back to
/// string equality.
fn values_equal(kind: &str, stored: &str, value: &str) -> bool {
    match kind {
        "entier" => match (stored.trim().parse::<i64>(), value.parse::<i64>()) {
            (Ok(a), Ok(b)) => a == b,
            _ => stored == value,
        },
        "reel" => match (stored.trim().parse::<f64>(), value.parse::<f64>()) {
            (Ok(a), Ok(b)) => a == b,
            _ => stored == value,
        },
        "booleen" => stored.trim().eq_ignore_ascii_case(value.trim()),
        _ => stored == value,
    }
}

fn split_assignments(tokens: &[Token]) -> Vec<&[Token]> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate() {
        if token.text == "[,]" {
            groups.push(&tokens[start..index]);
            start = index + 1;
        }
    }
    if start < tokens.len() {
        groups.push(&tokens[start..]);
    }
    groups
}

fn next_id(rows: &[Vec<String>], index: usize) -> i64 {
    rows.iter()
        .filter_map(|row| row.get(index))
        .filter_map(|value| value.trim().parse::<i64>().ok())
        .max()
        .unwrap_or(0)
        + 1
}

fn get_table(root: &Path, name: &str) -> Result<TableSchema, String> {
    let schema = storage::find_table(root, name)?
        .ok_or_else(|| format!("La table '{}' n'a pas été créée", name))?;
    if !storage::table_path(root, name)?.exists() {
        return Err(format!("La table '{}' n'a pas été créée", name));
    }
    Ok(schema)
}

/// Validates that every stored row matches the schema before rewriting a
/// table file, so corrupted data can never be silently rewritten.
fn check_rows_match_schema(
    rows: &[Vec<String>],
    schema: &TableSchema,
) -> Result<(), String> {
    for (number, row) in rows.iter().enumerate() {
        if row.len() != schema.columns.len() {
            return Err(format!(
                "La ligne {} de '{}.txt' ne correspond pas à la structure de manager.txt",
                number + 1,
                schema.name
            ));
        }
    }
    Ok(())
}

struct Condition {
    column: usize,
    op: String,
    value: String,
}

fn extract_conditions(
    tokens: &[Token],
    schema: &TableSchema,
) -> Result<(Vec<Condition>, Vec<String>), String> {
    let mut conds = Vec::new();
    let mut joiners = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if i + 3 > tokens.len() {
            return Err("Condition invalide".to_string());
        }
        let column_name = var_name(&tokens[i]);
        let Some(index) = schema
            .columns
            .iter()
            .position(|column| column.name == column_name)
        else {
            return Err(format!(
                "La colonne '${}' n'existe pas dans '{}'",
                column_name, schema.name
            ));
        };
        let kind = schema.columns[index].kind.clone();
        if !token_matches_strict(&tokens[i + 2], &kind) {
            return Err(type_error(&column_name, &kind));
        }
        conds.push(Condition {
            column: index,
            op: strip_brackets(&tokens[i + 1]).to_string(),
            value: stored_value(&kind, &tokens[i + 2]),
        });
        i += 3;
        if i < tokens.len() {
            joiners.push(strip_brackets(&tokens[i]).to_string());
            i += 1;
        }
    }
    Ok((conds, joiners))
}

fn compare(kind: &str, stored: &str, op: &str, value: &str) -> bool {
    let equal = values_equal(kind, stored, value);
    let (less, greater) = match kind {
        "entier" => match (stored.trim().parse::<i64>(), value.parse::<i64>()) {
            (Ok(a), Ok(b)) => (a < b, a > b),
            _ => (stored < value, stored > value),
        },
        "reel" => match (stored.trim().parse::<f64>(), value.parse::<f64>()) {
            (Ok(a), Ok(b)) => (a < b, a > b),
            _ => (stored < value, stored > value),
        },
        _ => (stored < value, stored > value),
    };
    match op {
        "=" => equal,
        "<>" | "!=" => !equal,
        "<" => less,
        "<=" => less || equal,
        ">" => greater,
        ">=" => greater || equal,
        _ => false,
    }
}

/// Evaluates a condition chain left-to-right: `et` binds the accumulator
/// with `&&`, `ou` with `||` (no precedence rules).
fn chain_matches(
    conds: &[Condition],
    joiners: &[String],
    schema: &TableSchema,
    row: &[String],
) -> bool {
    let eval = |index: usize| -> bool {
        let condition = &conds[index];
        let kind = &schema.columns[condition.column].kind;
        let stored = row.get(condition.column).map(|s| s.as_str()).unwrap_or("");
        compare(kind, stored, &condition.op, &condition.value)
    };

    let mut acc = !conds.is_empty() && eval(0);
    for (index, joiner) in joiners.iter().enumerate() {
        let next = eval(index + 1);
        acc = if joiner == "ET" { acc && next } else { acc || next };
    }
    acc
}

fn display_data(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let name = table_name(&tokens[3])?;
    let schema = get_table(root, &name)?;
    let columns = schema
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    let all_rows = storage::load_rows(root, &name)?;

    let rows = if tokens.len() == 4 {
        all_rows
    } else {
        let (conds, joiners) = extract_conditions(&tokens[5..], &schema)?;
        all_rows
            .into_iter()
            .filter(|row| chain_matches(&conds, &joiners, &schema, row))
            .collect::<Vec<_>>()
    };

    let mut result = QueryResult::success("AFFICHER", format!("{} ligne(s)", rows.len()));
    result.columns = Some(columns);
    result.row_count = Some(rows.len());
    result.rows = Some(rows);
    Ok(result)
}

fn insert_data(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let name = table_name(&tokens[2])?;
    let schema = get_table(root, &name)?;
    let columns = schema
        .columns
        .iter()
        .map(|column| column.name.as_str())
        .collect::<Vec<_>>();

    let mut provided: Vec<(String, String)> = Vec::new();
    for group in split_assignments(&tokens[4..]) {
        if group.len() != 3 {
            return Err("Affectation invalide".to_string());
        }
        let column = var_name(&group[0]);
        let Some(index) = columns.iter().position(|candidate| *candidate == column) else {
            return Err(format!(
                "La colonne '${}' n'existe pas dans '{}'",
                column, name
            ));
        };
        if provided.iter().any(|(existing, _)| existing == &column) {
            return Err(format!("La colonne '${}' est donnée deux fois", column));
        }
        if !token_matches_strict(&group[2], &schema.columns[index].kind) {
            return Err(type_error(&column, &schema.columns[index].kind));
        }
        provided.push((
            column,
            stored_value(&schema.columns[index].kind, &group[2]),
        ));
    }

    for column in &schema.columns {
        if provided.iter().any(|(existing, _)| existing == &column.name) {
            continue;
        }
        if column.name == "id" && column.kind == "entier" {
            continue;
        }
        return Err(format!("Valeur manquante pour '${}'", column.name));
    }

    let mut rows = storage::load_rows(root, &name)?;
    check_rows_match_schema(&rows, &schema)?;

    let mut new_row = Vec::with_capacity(schema.columns.len());
    for (index, column) in schema.columns.iter().enumerate() {
        if let Some((_, value)) = provided
            .iter()
            .find(|(existing, _)| existing == &column.name)
        {
            if column.name == "id"
                && rows.iter().any(|row| {
                    row.get(index)
                        .map(|stored| values_equal(&column.kind, stored, value))
                        .unwrap_or(false)
                })
            {
                return Err(format!("id '{}' existe déjà", value));
            }
            new_row.push(value.clone());
        } else {
            new_row.push(next_id(&rows, index).to_string());
        }
    }

    rows.push(new_row);
    storage::save_rows(root, &name, &rows)?;

    let mut result = QueryResult::success(
        "INSERER",
        format!("1 enregistrement ajouté dans '{}'", name),
    );
    result.affected = Some(1);
    result.row_count = Some(rows.len());
    Ok(result)
}

fn delete_data(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let name = table_name(&tokens[2])?;
    let schema = get_table(root, &name)?;

    let (conds, joiners) = extract_conditions(&tokens[4..], &schema)?;

    let rows = storage::load_rows(root, &name)?;
    check_rows_match_schema(&rows, &schema)?;

    let total = rows.len();
    let kept = rows
        .into_iter()
        .filter(|row| !chain_matches(&conds, &joiners, &schema, row))
        .collect::<Vec<_>>();
    let removed = total - kept.len();
    if removed > 0 {
        storage::save_rows(root, &name, &kept)?;
    }

    let mut result = QueryResult::success(
        "EFFACER",
        format!(
            "{} enregistrement(s) effacé(s) dans '{}'",
            removed, name
        ),
    );
    result.affected = Some(removed);
    result.row_count = Some(kept.len());
    Ok(result)
}

fn update_data(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let name = table_name(&tokens[1])?;
    let schema = get_table(root, &name)?;

    let Some(colon) = tokens.iter().position(|token| token.text == "[:]") else {
        return Err("':' attendu entre la condition et les affectations".to_string());
    };
    let (conds, joiners) = extract_conditions(&tokens[3..colon], &schema)?;

    let mut provided: Vec<(usize, String)> = Vec::new();
    for group in split_assignments(&tokens[colon + 1..]) {
        if group.len() != 3 {
            return Err("Affectation invalide".to_string());
        }
        let column = var_name(&group[0]);
        let Some(index) = schema
            .columns
            .iter()
            .position(|candidate| candidate.name == column)
        else {
            return Err(format!(
                "La colonne '${}' n'existe pas dans '{}'",
                column, name
            ));
        };
        if provided.iter().any(|(existing, _)| *existing == index) {
            return Err(format!("La colonne '${}' est donnée deux fois", column));
        }
        if !token_matches_strict(&group[2], &schema.columns[index].kind) {
            return Err(type_error(&column, &schema.columns[index].kind));
        }
        provided.push((
            index,
            stored_value(&schema.columns[index].kind, &group[2]),
        ));
    }

    let mut rows = storage::load_rows(root, &name)?;
    check_rows_match_schema(&rows, &schema)?;

    let matched = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| chain_matches(&conds, &joiners, &schema, row))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    let id_index = schema.columns.iter().position(|column| column.name == "id");
    if let Some(id_index) = id_index {
        if provided.iter().any(|(index, _)| *index == id_index) {
            if matched.len() > 1 {
                return Err(
                    "Impossible de modifier la colonne 'id' sur plusieurs lignes à la fois"
                        .to_string(),
                );
            }
            if let Some((_, value)) = provided.iter().find(|(index, _)| *index == id_index) {
                let kind = &schema.columns[id_index].kind;
                let conflicts = rows.iter().enumerate().any(|(row_index, row)| {
                    if matched.contains(&row_index) {
                        return false;
                    }
                    row.get(id_index)
                        .map(|stored| values_equal(kind, stored, value))
                        .unwrap_or(false)
                });
                if conflicts {
                    return Err(format!("id '{}' existe déjà", value));
                }
            }
        }
    }

    let mut changed = 0usize;
    for row_index in &matched {
        let row = &mut rows[*row_index];
        for (index, value) in &provided {
            row[*index] = value.clone();
        }
        changed += 1;
    }
    if changed > 0 {
        storage::save_rows(root, &name, &rows)?;
    }

    let mut result = QueryResult::success(
        "MODIFIER",
        format!(
            "{} enregistrement(s) modifié(s) dans '{}'",
            changed, name
        ),
    );
    result.affected = Some(changed);
    result.row_count = Some(rows.len());
    Ok(result)
}

fn create_file(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let name = table_name(&tokens[2])?;
    if storage::find_table(root, &name)?.is_some() {
        return Err(format!("La table '{}' existe déjà", name));
    }

    let mut columns = Vec::new();
    for field in split_assignments(&tokens[4..tokens.len() - 1]) {
        if field.len() != 3 {
            return Err("Colonne invalide".to_string());
        }
        let column_name = var_name(&field[0]);
        if columns
            .iter()
            .any(|column: &storage::Column| column.name == column_name)
        {
            return Err(format!("La colonne '${}' est donnée deux fois", column_name));
        }
        let kind = strip_brackets(&field[2]).to_lowercase();
        columns.push(storage::Column {
            name: column_name,
            kind,
        });
    }

    let schema = storage::create_table(root, &name, &columns)?;
    let mut result =
        QueryResult::success("CREER", format!("Table '{}' créée avec succès", name));
    result.columns = Some(
        schema
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect(),
    );
    result.rows = Some(Vec::new());
    result.row_count = Some(0);
    Ok(result)
}

fn drop_file(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let name = table_name(&tokens[2])?;
    storage::delete_table(root, &name)?;
    Ok(QueryResult::success(
        "SUPPRIMER",
        format!("Table '{}' supprimée", name),
    ))
}

fn rename_file(root: &Path, tokens: &[Token]) -> Result<QueryResult, String> {
    let from = table_name(&tokens[2])?;
    let to = table_name(&tokens[4])?;
    if from == to {
        return Err("Le nouveau nom doit être différent de l'ancien".to_string());
    }
    storage::rename_table(root, &from, &to)?;
    Ok(QueryResult::success(
        "RENOMMER",
        format!("Table '{}' renommée en '{}'", from, to),
    ))
}

pub fn run(root: &Path, query: &str) -> QueryResult {
    let started = Instant::now();
    let lexed = lexer::lex(query);

    if let Some(diagnostic) = lexed.errors.first() {
        return QueryResult::failure("", diagnostic.message.clone(), started);
    }

    let tokens = &lexed.tokens;
    if let Some(diagnostic) = validator::validate(tokens).first() {
        let command = command_of(tokens);
        return QueryResult::failure(&command, diagnostic.message.clone(), started);
    }

    let command = command_of(tokens);
    let executed = match tokens[0].text.as_str() {
        "[AFFICHER]" => display_data(root, tokens),
        "[INSERER]" => insert_data(root, tokens),
        "[EFFACER]" => delete_data(root, tokens),
        "[CREER]" => create_file(root, tokens),
        "[MODIFIER]" => update_data(root, tokens),
        "[SUPPRIMER]" => drop_file(root, tokens),
        "[RENOMMER]" => rename_file(root, tokens),
        _ => Err("Commande inconnue".to_string()),
    };

    match executed {
        Ok(mut result) => {
            result.elapsed_ms = started.elapsed().as_millis() as u64;
            result
        }
        Err(message) => QueryResult::failure(&command, message, started),
    }
}

/// Runs a whole editor document: statements are split on `;` or on newlines
/// (unless a parenthesised block or an unterminated string encloses them);
/// execution stops at the first failing statement.
pub fn run_script(root: &Path, query: &str) -> Vec<QueryResult> {
    let statements = super::split_statements(query);
    let mut results = Vec::new();
    for statement in statements {
        let result = run(root, &statement.text);
        let failed = !result.success;
        results.push(result);
        if failed {
            break;
        }
    }
    results
}
