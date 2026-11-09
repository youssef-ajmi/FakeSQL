use super::token::{Diagnostic, Token};

const BUILTINS: &[&str] = &[
    "[AFFICHER]",
    "[TOUS]",
    "[LES]",
    "[INSERER]",
    "[,]",
    "[DANS]",
    "[:]",
    "[EFFACER]",
    "[DE]",
    "[SI]",
    "[CREER]",
    "[FICHIER]",
    "[ENTIER]",
    "[CHAINE]",
    "[REEL]",
    "[BOOLEEN]",
    "[CARACTERE]",
    "[VRAI]",
    "[FAUX]",
    "[MODIFIER]",
    "[SUPPRIMER]",
    "[RENOMMER]",
    "[EN]",
    "[ET]",
    "[OU]",
    "[=]",
    "[<>]",
    "[!=]",
    "[<]",
    "[>]",
    "[<=]",
    "[>=]",
    "[(]",
    "[)]",
];

const VALUE_EXPECTED: &str =
    "Valeur attendue : entier, reel, vrai, faux ou chaîne 'entre guillemets'";

const COLUMN_EXPECTED: &str = "$colonne : entier | reel | booleen | caractere | chaine attendue";

const CREER_TYPES: &[&str] = &[
    "[ENTIER]",
    "[REEL]",
    "[BOOLEEN]",
    "[CARACTERE]",
    "[CHAINE]",
];

const CONDITION_OPERATORS: &[&str] = &["[=]", "[<>]", "[!=]", "[<]", "[>]", "[<=]", "[>=]"];

#[derive(Clone, Copy)]
struct Span {
    from: usize,
    to: usize,
}

impl Span {
    fn token(token: &Token) -> Self {
        Self {
            from: token.from,
            to: token.to,
        }
    }

    fn tokens(tokens: &[Token]) -> Self {
        match (tokens.first(), tokens.last()) {
            (Some(first), Some(last)) => Self {
                from: first.from,
                to: last.to,
            },
            _ => Self { from: 0, to: 0 },
        }
    }
}

fn error(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error(message, span.from, span.to)
}

fn is_quoted(token: &Token) -> bool {
    let bytes = token.text.as_bytes();
    bytes.len() >= 4 && bytes[1] == b'\'' && bytes[bytes.len() - 2] == b'\''
}

/// Bare numeric literal (entier or reel), e.g. `[12]` or `[3.5]`.
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

fn is_var(token: &Token) -> bool {
    let inner = &token.text[1..token.text.len() - 1];
    inner.len() > 1
        && inner.starts_with('$')
        && inner[1..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_value(token: &Token) -> bool {
    is_quoted(token) || is_bool(token) || is_number(token)
}

fn is_allowed(token: &Token) -> bool {
    BUILTINS.contains(&token.text.as_str())
        || is_quoted(token)
        || token.text.as_bytes().get(1) == Some(&b'$')
        || is_number(token)
}

fn split_by<'a>(tokens: &'a [Token], separator: &str) -> Vec<&'a [Token]> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate() {
        if token.text == separator {
            groups.push(&tokens[start..index]);
            start = index + 1;
        }
    }
    groups.push(&tokens[start..]);
    groups
}

fn expect_keyword(token: &Token, expected: &str) -> Result<(), Diagnostic> {
    if token.text == expected {
        Ok(())
    } else {
        Err(error(
            format!(
                "Mot-clé '{}' attendu à la place de '{}'",
                expected.trim_matches(&['[', ']'][..]).to_lowercase(),
                token.raw
            ),
            Span::token(token),
        ))
    }
}

fn expect_quoted(token: &Token) -> Result<(), Diagnostic> {
    if is_quoted(token) {
        Ok(())
    } else {
        Err(error(
            "Le nom de table doit être entre guillemets, ex. 'users'",
            Span::token(token),
        ))
    }
}

fn verify_condition_chain(tokens: &[Token]) -> Result<(), Diagnostic> {
    let mut i = 0;
    loop {
        if i + 3 > tokens.len() {
            return Err(error(
                "Condition attendue : $colonne = valeur [et|ou ...]",
                Span::tokens(&tokens[i.min(tokens.len())..]),
            ));
        }
        if !is_var(&tokens[i]) {
            return Err(error(
                "Condition attendue : $colonne = valeur (ex. $id = 1)",
                Span::token(&tokens[i]),
            ));
        }
        if !CONDITION_OPERATORS.contains(&tokens[i + 1].text.as_str()) {
            return Err(error(
                "Opérateur attendu : =, <>, <=, >=, <, >",
                Span::token(&tokens[i + 1]),
            ));
        }
        if !is_value(&tokens[i + 2]) {
            return Err(error(VALUE_EXPECTED, Span::token(&tokens[i + 2])));
        }
        i += 3;
        if i >= tokens.len() {
            return Ok(());
        }
        let joiner = &tokens[i];
        if joiner.text != "[ET]" && joiner.text != "[OU]" {
            return Err(error(
                format!(
                    "Mot-clé 'et' ou 'ou' attendu à la place de '{}'",
                    joiner.raw
                ),
                Span::token(joiner),
            ));
        }
        i += 1;
    }
}

fn verify_assignments(tokens: &[Token]) -> Result<(), Diagnostic> {
    let groups = split_by(tokens, "[,]");
    if groups.is_empty() {
        return Err(error(
            "Au moins une affectation est attendue",
            Span::tokens(tokens),
        ));
    }

    for group in groups {
        if group.is_empty() {
            return Err(error(
                "Affectation vide : $colonne = valeur attendue",
                Span::tokens(tokens),
            ));
        }
        if group.len() != 3 {
            return Err(error(
                "Affectation invalide : $colonne = valeur attendue",
                Span::tokens(group),
            ));
        }
        if !is_var(&group[0]) {
            return Err(error(
                "Variable attendue (ex. $username)",
                Span::token(&group[0]),
            ));
        }
        expect_keyword(&group[1], "[=]")?;
        if !is_value(&group[2]) {
            return Err(error(VALUE_EXPECTED, Span::token(&group[2])));
        }
    }
    Ok(())
}

fn verify_afficher(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() == 4 {
        expect_keyword(&tokens[1], "[TOUS]")?;
        expect_keyword(&tokens[2], "[LES]")?;
        return expect_quoted(&tokens[3]);
    }

    if tokens.len() >= 8 {
        expect_keyword(&tokens[1], "[TOUS]")?;
        expect_keyword(&tokens[2], "[LES]")?;
        expect_quoted(&tokens[3])?;
        expect_keyword(&tokens[4], "[SI]")?;
        return verify_condition_chain(&tokens[5..]);
    }

    Err(error(
        "Syntaxe attendue : afficher tous les 'table' [si $colonne = valeur]",
        Span::tokens(tokens),
    ))
}

fn verify_inserer(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() < 7 {
        return Err(error(
            "Syntaxe attendue : inserer dans 'table' : $col = valeur, ...",
            Span::tokens(tokens),
        ));
    }
    expect_keyword(&tokens[1], "[DANS]")?;
    expect_quoted(&tokens[2])?;
    expect_keyword(&tokens[3], "[:]")?;
    verify_assignments(&tokens[4..])
}

fn verify_creer(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() < 8 {
        return Err(error(
            "Syntaxe attendue : creer fichier 'table' ( $col : entier , ... )",
            Span::tokens(tokens),
        ));
    }
    expect_keyword(&tokens[1], "[FICHIER]")?;
    expect_quoted(&tokens[2])?;
    expect_keyword(&tokens[3], "[(]")?;
    let last = tokens.last().expect("tokens is not empty");
    expect_keyword(last, "[)]")?;

    let fields = split_by(&tokens[4..tokens.len() - 1], "[,]");
    if fields.is_empty() {
        return Err(error(
            "Au moins une colonne est attendue",
            Span::tokens(tokens),
        ));
    }

    let mut seen: Vec<String> = Vec::new();
    for field in fields {
        if field.is_empty() {
            return Err(error(
                format!("Colonne vide : {COLUMN_EXPECTED}"),
                Span::tokens(tokens),
            ));
        }
        if field.len() != 3 {
            return Err(error(
                format!("Colonne invalide : {COLUMN_EXPECTED}"),
                Span::tokens(field),
            ));
        }
        if !is_var(&field[0]) {
            return Err(error(
                "Variable attendue (ex. $username)",
                Span::token(&field[0]),
            ));
        }
        let name = field[0].text[2..field[0].text.len() - 1].to_string();
        if seen.contains(&name) {
            return Err(error(
                format!("La colonne '{}' est donnée deux fois", name),
                Span::tokens(field),
            ));
        }
        seen.push(name);
        expect_keyword(&field[1], "[:]")?;
        if !CREER_TYPES.contains(&field[2].text.as_str()) {
            return Err(error(
                "Type attendu : entier, reel, booleen, caractere ou chaine",
                Span::token(&field[2]),
            ));
        }
    }
    Ok(())
}

fn verify_effacer(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() < 7 {
        return Err(error(
            "Syntaxe attendue : effacer de 'table' si $colonne = valeur",
            Span::tokens(tokens),
        ));
    }
    expect_keyword(&tokens[1], "[DE]")?;
    expect_quoted(&tokens[2])?;
    expect_keyword(&tokens[3], "[SI]")?;
    verify_condition_chain(&tokens[4..])
}

fn verify_modifier(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() < 10 {
        return Err(error(
            "Syntaxe attendue : modifier 'table' si $col = valeur : $col = valeur",
            Span::tokens(tokens),
        ));
    }
    expect_quoted(&tokens[1])?;
    expect_keyword(&tokens[2], "[SI]")?;

    let Some(colon) = tokens
        .iter()
        .skip(3)
        .position(|token| token.text == "[:]")
        .map(|offset| offset + 3)
    else {
        return Err(error(
            "':' attendu entre la condition et les affectations",
            Span::tokens(tokens),
        ));
    };

    let chain = &tokens[3..colon];
    if chain.is_empty() {
        return Err(error(
            "Condition attendue : $colonne = valeur",
            Span::tokens(tokens),
        ));
    }
    verify_condition_chain(chain)?;
    verify_assignments(&tokens[colon + 1..])
}

fn verify_supprimer(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() != 3 {
        return Err(error(
            "Syntaxe attendue : supprimer fichier 'table'",
            Span::tokens(tokens),
        ));
    }
    expect_keyword(&tokens[1], "[FICHIER]")?;
    expect_quoted(&tokens[2])
}

fn verify_renommer(tokens: &[Token]) -> Result<(), Diagnostic> {
    if tokens.len() != 5 {
        return Err(error(
            "Syntaxe attendue : renommer fichier 'ancien' en 'nouveau'",
            Span::tokens(tokens),
        ));
    }
    expect_keyword(&tokens[1], "[FICHIER]")?;
    expect_quoted(&tokens[2])?;
    expect_keyword(&tokens[3], "[EN]")?;
    expect_quoted(&tokens[4])
}

pub fn validate(tokens: &[Token]) -> Vec<Diagnostic> {
    if tokens.is_empty() {
        return vec![Diagnostic::error("Requête vide", 0, 0)];
    }

    if let Some(token) = tokens.iter().find(|token| !is_allowed(token)) {
        return vec![error(
            format!("Symbole inattendu '{}'", token.raw),
            Span::token(token),
        )];
    }

    let result = match tokens[0].text.as_str() {
        "[AFFICHER]" => verify_afficher(tokens),
        "[INSERER]" => verify_inserer(tokens),
        "[EFFACER]" => verify_effacer(tokens),
        "[CREER]" => verify_creer(tokens),
        "[MODIFIER]" => verify_modifier(tokens),
        "[SUPPRIMER]" => verify_supprimer(tokens),
        "[RENOMMER]" => verify_renommer(tokens),
        other => Err(error(
            format!(
                "Commande inconnue '{}'",
                other.trim_matches(&['[', ']'][..]).to_lowercase()
            ),
            Span::token(&tokens[0]),
        )),
    };

    result.err().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::super::lexer::lex;
    use super::validate;

    fn errors(query: &str) -> Vec<String> {
        let result = lex(query);
        if !result.errors.is_empty() {
            return result.errors.into_iter().map(|e| e.message).collect();
        }
        validate(&result.tokens)
            .into_iter()
            .map(|e| e.message)
            .collect()
    }

    #[test]
    fn accepts_all_commands() {
        for query in [
            "afficher tous les 'users'",
            "afficher tous les 'users' si $id = 4",
            "afficher tous les 'users' si $name = 'aa' et $id <> 3",
            "inserer dans 'users' : $username = 'aa' , $password = '0000'",
            "inserer dans 'users' : $id = 4 , $username = 'bb' , $password = '1234'",
            "effacer de 'users' si $id = 123",
            "effacer de 'users' si $id = 1 ou $name = 'aa'",
            "creer fichier 'users' ( $id : entier , $username : chaine , $password : chaine )",
            "modifier 'users' si $id = 1 : $password = '9999' , $username = 'zz'",
            "supprimer fichier 'users'",
            "renommer fichier 'users' en 'clients'",
        ] {
            assert!(errors(query).is_empty(), "query failed: {query}");
        }
    }

    #[test]
    fn rejects_bad_commands() {
        for query in [
            "afficher les 'users'",
            "afficher tous les 'users' si $id ! 4",
            "afficher tous les 'users' si $id = 4 $name = 'a'",
            "inserer dans 'users' : $username 'aa'",
            "inserer dans 'users' : username = 'aa'",
            "effacer de 'users' si $id 123",
            "creer fichier 'users' ( $id : nombre )",
            "creer fichier 'users' ( $id : entier , $id : chaine )",
            "modifier 'users' $id = 1 : $name = 'x'",
            "modifier 'users' si $id = 1 : $name",
            "supprimer 'users'",
            "renommer fichier 'a' vers 'b'",
        ] {
            assert!(!errors(query).is_empty(), "query should fail: {query}");
        }
    }

    #[test]
    fn detects_duplicate_columns_in_creer() {
        let messages = errors("creer fichier 'users' ( $id : entier , $id : chaine )");
        assert!(messages.iter().any(|m| m.contains("donnée deux fois")));
    }

    #[test]
    fn accepts_new_types_and_literals() {
        assert!(errors(
            "creer fichier 'stats' ( $t : reel , $b : booleen , $c : caractere , $n : chaine )"
        )
        .is_empty());
        assert!(errors("effacer de 'stats' si $b = vrai").is_empty());
        assert!(errors("effacer de 'stats' si $b = faux ou $t = 1.5").is_empty());
        assert!(errors("inserer dans 'stats' : $t = 3.5 , $c = 'A' , $n = 'aa'").is_empty());
    }

    #[test]
    fn rejects_bad_type_names_and_bad_literals() {
        // An unknown bare word is caught by the generic symbol check.
        let messages = errors("creer fichier 'stats' ( $t : numerique )");
        assert!(messages.iter().any(|m| m.contains("Symbole inattendu")));

        // A globally valid token used as a type gets the targeted message.
        let messages = errors("creer fichier 'stats' ( $t : vrai )");
        assert!(messages.iter().any(|m| m.contains("Type attendu")));

        // Reel literals parse as values but not as column names.
        assert!(!errors("creer fichier 'stats' ( 3.5 : reel )").is_empty());
    }
}
