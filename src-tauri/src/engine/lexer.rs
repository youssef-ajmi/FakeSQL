use super::token::{Diagnostic, Token};

pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<Diagnostic>,
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn to_token(raw: &str) -> String {
    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        format!("[{}]", raw)
    } else if raw.starts_with('$') {
        format!("[{}]", raw)
    } else {
        format!("[{}]", raw.to_uppercase())
    }
}

/// Total UTF-16 code-unit length of a string, used for token positions so the
/// frontend can map diagnostics onto CodeMirror documents directly.
fn utf16_len(input: &str) -> usize {
    input.chars().map(char::len_utf16).sum()
}

pub fn lex(input: &str) -> LexResult {
    let chars: Vec<(usize, usize, char)> = input
        .char_indices()
        .map(|(byte, c)| (byte, c.len_utf16(), c))
        .collect();
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut i = 0;
    // UTF-16 offset of the current char, in sync with `i`.
    let mut utf16 = 0usize;

    fn width_of(chars: &[(usize, usize, char)], from: usize, to: usize) -> usize {
        chars[from..to].iter().map(|(_, width, _)| *width).sum()
    }

    while i < chars.len() {
        let (_, width16, c) = chars[i];
        let start_u = utf16;

        if c.is_whitespace() {
            utf16 += width16;
            i += 1;
            continue;
        }

        if c == '#' {
            // Comment to the end of the line: consume it entirely.
            while i < chars.len() && chars[i].2 != '\n' {
                utf16 += chars[i].1;
                i += 1;
            }
            continue;
        }

        if c == '/' {
            let next = chars.get(i + 1).map(|(_, _, c)| *c);
            if next == Some('/') {
                while i < chars.len() && chars[i].2 != '\n' {
                    utf16 += chars[i].1;
                    i += 1;
                }
                continue;
            }
            errors.push(Diagnostic::error(
                format!("Caractère inattendu '{}'", c),
                start_u,
                start_u + width16,
            ));
            break;
        }

        if c == '\'' {
            let mut j = i + 1;
            let mut closed = false;
            while j < chars.len() {
                if chars[j].2 == '\'' {
                    // '' is an escaped apostrophe: both quotes stay inside
                    // the string, only a lone ' closes the literal.
                    if chars.get(j + 1).map(|(_, _, c)| *c) == Some('\'') {
                        j += 2;
                        continue;
                    }
                    closed = true;
                    break;
                }
                j += 1;
            }

            if !closed {
                errors.push(Diagnostic::error(
                    "Chaîne de caractères non terminée",
                    start_u,
                    utf16_len(input),
                ));
                break;
            }

            let to = utf16 + width_of(&chars, i, j + 1);
            let raw = &input[chars[i].0..chars[j].0 + chars[j].2.len_utf8()];
            tokens.push(Token {
                text: to_token(raw),
                raw: raw.to_string(),
                from: start_u,
                to,
            });
            utf16 = to;
            i = j + 1;
            continue;
        }

        if c == '$' {
            let mut j = i + 1;
            while j < chars.len() && is_word_char(chars[j].2) {
                j += 1;
            }

            if j == i + 1 {
                errors.push(Diagnostic::error(
                    "Variable invalide : un nom est attendu après '$'",
                    start_u,
                    start_u + width16,
                ));
                utf16 += width16;
                i += 1;
                continue;
            }

            let to = utf16 + width_of(&chars, i, j);
            let end = if j < chars.len() { chars[j].0 } else { input.len() };
            let raw = &input[chars[i].0..end];
            tokens.push(Token {
                text: to_token(raw),
                raw: raw.to_string(),
                from: start_u,
                to,
            });
            utf16 = to;
            i = j;
            continue;
        }

        if c.is_ascii_digit() {
            // Bare number: integer `12` or real `3.5` (digits on both sides
            // of the point; `5.` is invalid).
            let mut j = i + 1;
            while j < chars.len() && chars[j].2.is_ascii_digit() {
                j += 1;
            }
            if j < chars.len() && chars[j].2 == '.' {
                if j + 1 < chars.len() && chars[j + 1].2.is_ascii_digit() {
                    j += 2;
                    while j < chars.len() && chars[j].2.is_ascii_digit() {
                        j += 1;
                    }
                } else {
                    errors.push(Diagnostic::error(
                        "Nombre invalide : chiffres attendus après le point",
                        start_u,
                        utf16 + width_of(&chars, i, j + 1),
                    ));
                    break;
                }
            }

            let to = utf16 + width_of(&chars, i, j);
            let end = if j < chars.len() { chars[j].0 } else { input.len() };
            let raw = &input[chars[i].0..end];
            tokens.push(Token {
                text: to_token(raw),
                raw: raw.to_string(),
                from: start_u,
                to,
            });
            utf16 = to;
            i = j;
            continue;
        }

        if is_word_char(c) {
            let mut j = i + 1;
            while j < chars.len() && is_word_char(chars[j].2) {
                j += 1;
            }
            let to = utf16 + width_of(&chars, i, j);
            let end = if j < chars.len() { chars[j].0 } else { input.len() };
            let raw = &input[chars[i].0..end];
            tokens.push(Token {
                text: to_token(raw),
                raw: raw.to_string(),
                from: start_u,
                to,
            });
            utf16 = to;
            i = j;
            continue;
        }

        if matches!(c, '(' | ')' | ',' | ':' | '=') {
            tokens.push(Token {
                text: to_token(&c.to_string()),
                raw: c.to_string(),
                from: start_u,
                to: start_u + width16,
            });
            utf16 += width16;
            i += 1;
            continue;
        }

        if c == '<' || c == '>' || c == '!' {
            let next = chars.get(i + 1).map(|(_, _, c)| *c);
            let text = match (c, next) {
                ('<', Some('>')) => Some("[<>]"),
                ('<', Some('=')) => Some("[<=]"),
                ('>', Some('=')) => Some("[>=]"),
                ('!', Some('=')) => Some("[!=]"),
                ('<', _) => Some("[<]"),
                ('>', _) => Some("[>]"),
                _ => None,
            };

            let Some(text) = text else {
                errors.push(Diagnostic::error(
                    format!("Caractère inattendu '{}'", c),
                    start_u,
                    start_u + width16,
                ));
                break;
            };

            let span = if next == Some('=') || (c, next) == ('<', Some('>')) {
                2
            } else {
                1
            };
            tokens.push(Token {
                text: text.to_string(),
                raw: c.to_string(),
                from: start_u,
                to: start_u + span,
            });
            utf16 += width_of(&chars, i, i + span);
            i += span;
            continue;
        }

        errors.push(Diagnostic::error(
            format!("Caractère inattendu '{}'", c),
            start_u,
            start_u + width16,
        ));
        i += 1;
        utf16 += width16;
    }

    LexResult { tokens, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_afficher() {
        let result = lex("afficher tous les 'users'");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["[AFFICHER]", "[TOUS]", "[LES]", "['users']"]);
    }

    #[test]
    fn tokenizes_variables_and_punctuation() {
        let result = lex("inserer dans 'users' : $id = 12 , $name = 'aa'");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "[INSERER]", "[DANS]", "['users']", "[:]", "[$id]", "[=]", "[12]",
                "[,]", "[$name]", "[=]", "['aa']"
            ]
        );
    }

    #[test]
    fn reports_unterminated_string() {
        let result = lex("afficher tous les 'users");
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].severity, "error");
    }

    #[test]
    fn uses_utf16_offsets() {
        let result = lex("afficher 'é' tous");
        assert!(result.errors.is_empty());
        let tokens = &result.tokens;
        assert_eq!(tokens[0].text, "[AFFICHER]");
        assert_eq!(tokens[0].from, 0);
        assert_eq!(tokens[0].to, 8);
        assert_eq!(tokens[1].text, "['é']");
        // 9 (start) + 3 UTF-16 units (' é ') — byte offsets would be 9..11.
        assert_eq!(tokens[1].from, 9);
        assert_eq!(tokens[1].to, 12);
        assert_eq!(tokens[2].text, "[TOUS]");
        assert_eq!(tokens[2].from, 13);
        assert_eq!(tokens[2].to, 17);
    }

    #[test]
    fn skips_comments() {
        let result = lex("# note\nafficher tous les 'users' // inline");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["[AFFICHER]", "[TOUS]", "[LES]", "['users']"]);
    }

    #[test]
    fn tokenizes_operators() {
        let result = lex("si $a <= 1 et $b <> 2 ou $c != 'x' et $d > 3 et $e >= 4 et $f < 5");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "[SI]", "[$a]", "[<=]", "[1]", "[ET]", "[$b]", "[<>]", "[2]", "[OU]",
                "[$c]", "[!=]", "['x']", "[ET]", "[$d]", "[>]", "[3]", "[ET]",
                "[$e]", "[>=]", "[4]", "[ET]", "[$f]", "[<]", "[5]"
            ]
        );
    }

    #[test]
    fn reports_bare_bang() {
        let result = lex("si $a ! 1");
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].message, "Caractère inattendu '!'");
    }

    #[test]
    fn tokenizes_reals() {
        let result = lex("si $prix > 10.50");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["[SI]", "[$prix]", "[>]", "[10.50]"]);
    }

    #[test]
    fn rejects_trailing_point_in_number() {
        let result = lex("si $prix = 5.");
        assert_eq!(result.errors.len(), 1);
        assert_eq!(
            result.errors[0].message,
            "Nombre invalide : chiffres attendus après le point"
        );
    }

    #[test]
    fn keeps_doubled_quote_inside_string() {
        let result = lex("afficher tous les 'x' si $t = 'it''s'");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["[AFFICHER]", "[TOUS]", "[LES]", "['x']", "[SI]", "[$t]", "[=]", "['it''s']"]
        );
        // The value keeps the escape encoded in `text`/`raw`.
        let last = result.tokens.last().unwrap();
        assert_eq!(last.text, "['it''s']");
        assert_eq!(last.raw, "'it''s'");
    }

    #[test]
    fn accepts_empty_string() {
        let result = lex("inserer dans 'x' : $t = ''");
        assert!(result.errors.is_empty());
        let texts: Vec<&str> = result.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["[INSERER]", "[DANS]", "['x']", "[:]", "[$t]", "[=]", "['']"]);
    }
}
