pub mod executor;
pub mod lexer;
pub mod storage;
pub mod token;
pub mod validator;

use serde::Serialize;

use token::Diagnostic;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<Diagnostic>,
}

/// One runnable statement extracted from an editor document.
#[derive(Debug, Clone)]
pub struct Statement {
    pub text: String,
    /// UTF-16 offset of this statement's first character in the parent text.
    pub utf16_start: usize,
}

/// Splits a multi-statement document. A statement ends at `;` or at a newline,
/// unless the newline sits inside a quoted string or inside a parenthesised
/// block (so long `creer fichier (...)` statements can wrap across lines).
/// `#` and `//` start line comments. Statement text keeps its leading
/// whitespace so local UTF-16 offsets map onto the document unchanged.
pub fn split_statements(input: &str) -> Vec<Statement> {
    let mut statements: Vec<Statement> = Vec::new();
    let mut in_string = false;
    let mut depth: i32 = 0;
    let mut start_byte = 0usize;
    let mut start_utf16 = 0usize;
    let mut utf16 = 0usize;

    let push = |statements: &mut Vec<Statement>, slice: &str, start: usize| {
        if slice.trim().is_empty() {
            return;
        }
        let lexed = lexer::lex(slice);
        // Comment-only statements produce no tokens (and no errors); they are
        // boundaries, not runnable code.
        if lexed.tokens.is_empty() && lexed.errors.is_empty() {
            return;
        }
        statements.push(Statement {
            text: slice.to_string(),
            utf16_start: start,
        });
    };

    let mut iterator = input.char_indices().peekable();
    while let Some((byte, c)) = iterator.next() {
        match c {
            '\'' => {
                utf16 += 1;
                if in_string {
                    // '' is an escaped apostrophe: both quotes are consumed
                    // and the string stays open. A lone ' closes it.
                    match iterator.peek() {
                        Some((_, '\'')) => {
                            iterator.next();
                            utf16 += 1;
                        }
                        _ => in_string = false,
                    }
                } else {
                    in_string = true;
                }
            }
            '(' => {
                if !in_string {
                    depth += 1;
                }
                utf16 += c.len_utf16();
            }
            ')' => {
                if !in_string && depth > 0 {
                    depth -= 1;
                }
                utf16 += c.len_utf16();
            }
            ';' => {
                let width = c.len_utf16();
                if !in_string {
                    push(&mut statements, &input[start_byte..byte], start_utf16);
                    start_byte = byte + c.len_utf8();
                    start_utf16 = utf16 + width;
                }
                utf16 += width;
            }
            '\n' => {
                let width = c.len_utf16();
                if !in_string && depth == 0 {
                    push(&mut statements, &input[start_byte..byte], start_utf16);
                    start_byte = byte + c.len_utf8();
                    start_utf16 = utf16 + width;
                }
                utf16 += width;
            }
            '#' => {
                utf16 += c.len_utf16();
                if !in_string {
                    let mut consumed_newline_utf16 = 0usize;
                    let mut newline_byte = None;
                    while let Some((inner_byte, nc)) = iterator.next() {
                        utf16 += nc.len_utf16();
                        if nc == '\n' {
                            consumed_newline_utf16 = nc.len_utf16();
                            newline_byte = Some(inner_byte);
                            break;
                        }
                    }
                    if let Some(newline_byte) = newline_byte {
                        push(&mut statements, &input[start_byte..newline_byte], start_utf16);
                        start_byte = newline_byte + 1;
                        start_utf16 = utf16;
                        let _ = consumed_newline_utf16;
                    }
                }
            }
            '/' if !in_string => {
                let is_comment = iterator
                    .peek()
                    .map(|(_, next)| *next == '/')
                    .unwrap_or(false);
                utf16 += c.len_utf16();
                if is_comment {
                    iterator.next();
                    while let Some((inner_byte, nc)) = iterator.next() {
                        utf16 += nc.len_utf16();
                        if nc == '\n' {
                            push(&mut statements, &input[start_byte..inner_byte], start_utf16);
                            start_byte = inner_byte + 1;
                            start_utf16 = utf16;
                            break;
                        }
                    }
                }
            }
            _ => {
                utf16 += c.len_utf16();
            }
        }
    }
    if start_byte < input.len() {
        push(&mut statements, &input[start_byte..], start_utf16);
    }

    statements
}

fn shift_diagnostic(diagnostic: Diagnostic, offset: usize) -> Diagnostic {
    Diagnostic {
        message: diagnostic.message,
        from: diagnostic.from.saturating_add(offset),
        to: diagnostic.to.saturating_add(offset),
        severity: diagnostic.severity,
    }
}

/// Validates every statement of a document; diagnostics are reported with
/// document-wide UTF-16 offsets so the editor can underline the right text.
pub fn validate_query(query: &str) -> ValidationResult {
    if query.trim().is_empty() {
        return ValidationResult {
            valid: false,
            errors: vec![Diagnostic::error("Requête vide", 0, 0)],
        };
    }

    let mut errors = Vec::new();
    for statement in split_statements(query) {
        let lexed = lexer::lex(&statement.text);
        if !lexed.errors.is_empty() {
            errors.extend(
                lexed
                    .errors
                    .into_iter()
                    .map(|diagnostic| shift_diagnostic(diagnostic, statement.utf16_start)),
            );
            continue;
        }
        errors.extend(
            validator::validate(&lexed.tokens)
                .into_iter()
                .map(|diagnostic| shift_diagnostic(diagnostic, statement.utf16_start)),
        );
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}
