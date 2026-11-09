use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use fakesql_studio_lib::engine::executor::{self, run_script};
use fakesql_studio_lib::engine::storage::workspace_issues;

fn run(root: &PathBuf, query: &str) -> executor::QueryResult {
    executor::run(root, query)
}

fn workspace() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fakesql-studio-test-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn seed_users(root: &PathBuf) {
    let created = run(
        root,
        "creer fichier 'users' ( $id : entier , $name : chaine , $age : entier )",
    );
    assert!(created.success, "{:?}", created.message);
    for (id, name, age) in [(1, "'aa'", 20), (2, "'bb'", 30), (3, "'cc'", 40)] {
        let inserted = run(
            root,
            &format!("inserer dans 'users' : $id = {} , $name = {} , $age = {}", id, name, age),
        );
        assert!(inserted.success, "{:?}", inserted.message);
    }
}

#[test]
fn full_lifecycle() {
    let root = workspace();

    let created = run(
        &root,
        "creer fichier 'users' ( $id : entier , $username : chaine , $password : chaine )",
    );
    assert!(created.success, "{:?}", created.message);

    let inserted = run(
        &root,
        "inserer dans 'users' : $username = 'aa' , $password = '0000'",
    );
    assert!(inserted.success, "{:?}", inserted.message);
    assert_eq!(inserted.affected, Some(1));

    let explicit = run(
        &root,
        "inserer dans 'users' : $id = 5 , $username = 'bb' , $password = '1234'",
    );
    assert!(explicit.success, "{:?}", explicit.message);

    let displayed = run(&root, "afficher tous les 'users'");
    assert!(displayed.success, "{:?}", displayed.message);
    assert_eq!(displayed.columns.as_ref().unwrap().len(), 3);
    let rows = displayed.rows.as_ref().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["1", "aa", "0000"]);
    assert_eq!(rows[1], vec!["5", "bb", "1234"]);

    let deleted = run(&root, "effacer de 'users' si $id = 1");
    assert!(deleted.success, "{:?}", deleted.message);
    assert_eq!(deleted.affected, Some(1));

    let displayed = run(&root, "afficher tous les 'users'");
    assert_eq!(displayed.rows.as_ref().unwrap().len(), 1);

    // The catalog no longer stores a row-count cache.
    let manager = fs::read_to_string(root.join("manager.txt")).unwrap();
    assert_eq!(
        manager.trim(),
        "users.txt > id:entier|username:chaine|password:chaine"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_invalid_data() {
    let root = workspace();

    let created = run(&root, "creer fichier 'users' ( $id : entier , $name : chaine )");
    assert!(created.success, "{:?}", created.message);

    let missing = run(&root, "inserer dans 'users' : $id = 1");
    assert!(!missing.success);
    assert!(missing.message.unwrap().contains("Valeur manquante"));

    let wrong_type = run(
        &root,
        "inserer dans 'users' : $id = 1 , $name = 12",
    );
    assert!(!wrong_type.success);
    assert!(wrong_type.message.unwrap().contains("doit être de type chaine"));

    let good = run(
        &root,
        "inserer dans 'users' : $id = 1 , $name = 'first'",
    );
    assert!(good.success, "{:?}", good.message);

    let duplicate = run(
        &root,
        "inserer dans 'users' : $id = 1 , $name = 'second'",
    );
    assert!(!duplicate.success);
    assert!(duplicate.message.unwrap().contains("existe déjà"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn enforces_strict_typing() {
    let root = workspace();

    run(&root, "creer fichier 'users' ( $id : entier , $name : chaine )");
    let quoted_id = run(
        &root,
        "inserer dans 'users' : $id = '1' , $name = 'first'",
    );
    assert!(!quoted_id.success);
    assert!(quoted_id.message.unwrap().contains("doit être de type entier"));

    let bare_id = run(
        &root,
        "inserer dans 'users' : $id = 1 , $name = 'first'",
    );
    assert!(bare_id.success, "{:?}", bare_id.message);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn numeric_id_comparison_ignores_zero_padding() {
    let root = workspace();

    run(&root, "creer fichier 'users' ( $id : entier , $name : chaine )");
    run(&root, "inserer dans 'users' : $id = 1 , $name = 'first'");
    let removed = run(&root, "effacer de 'users' si $id = 01");
    assert!(removed.success, "{:?}", removed.message);
    assert_eq!(removed.affected, Some(1));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_bad_syntax_without_touching_files() {
    let root = workspace();

    let result = run(&root, "afficher les 'users'");
    assert!(!result.success);
    assert!(!root.join("users.txt").exists());

    let result = run(&root, "supprimer 'users'");
    assert!(!result.success);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn displays_existing_sample_rows() {
    let root = workspace();
    fs::write(
        root.join("manager.txt"),
        "users.txt > id:entier|username:chaine|password:chaine*2\n",
    )
    .unwrap();
    fs::write(root.join("users.txt"), "1|youssef|0000\n3|aaeeee|1retrh\n").unwrap();

    let displayed = run(&root, "afficher tous les 'users'");
    assert!(displayed.success, "{:?}", displayed.message);
    assert_eq!(displayed.rows.as_ref().unwrap().len(), 2);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn filters_afficher_and_effacer_with_operators() {
    let root = workspace();
    seed_users(&root);

    let filtered = run(&root, "afficher tous les 'users' si $age > 25");
    assert!(filtered.success, "{:?}", filtered.message);
    assert_eq!(filtered.row_count, Some(2));
    let rows = filtered.rows.as_ref().unwrap();
    assert_eq!(rows[0], vec!["2", "bb", "30"]);
    assert_eq!(rows[1], vec!["3", "cc", "40"]);

    let by_name = run(&root, "afficher tous les 'users' si $name = 'aa'");
    assert_eq!(by_name.row_count, Some(1));

    let removed = run(&root, "effacer de 'users' si $age = 20 ou $age = 30");
    assert!(removed.success, "{:?}", removed.message);
    assert_eq!(removed.affected, Some(2));

    let remaining = run(&root, "afficher tous les 'users'");
    assert_eq!(remaining.row_count, Some(1));
    assert_eq!(remaining.rows.as_ref().unwrap()[0], vec!["3", "cc", "40"]);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn update_modifies_matching_rows() {
    let root = workspace();
    seed_users(&root);

    let updated = run(&root, "modifier 'users' si $id = 2 : $name = 'bbb' , $age = 31");
    assert!(updated.success, "{:?}", updated.message);
    assert_eq!(updated.affected, Some(1));

    let displayed = run(&root, "afficher tous les 'users' si $id = 2");
    let rows = displayed.rows.as_ref().unwrap();
    assert_eq!(rows[0], vec!["2", "bbb", "31"]);

    let unchanged = run(&root, "afficher tous les 'users' si $name = 'aa'");
    assert_eq!(unchanged.row_count, Some(1));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn update_rejects_conflicting_id_changes() {
    let root = workspace();
    seed_users(&root);

    let conflict = run(&root, "modifier 'users' si $id = 1 : $id = 3");
    assert!(!conflict.success);
    assert!(conflict.message.unwrap().contains("existe déjà"));

    let bulk_id = run(&root, "modifier 'users' si $age > 25 : $id = 9");
    assert!(!bulk_id.success);
    assert!(bulk_id.message.unwrap().contains("plusieurs lignes"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn drop_and_rename_tables() {
    let root = workspace();
    seed_users(&root);

    let renamed = run(&root, "renommer fichier 'users' en 'people'");
    assert!(renamed.success, "{:?}", renamed.message);
    assert!(!root.join("users.txt").exists());
    assert!(root.join("people.txt").exists());
    let manager = fs::read_to_string(root.join("manager.txt")).unwrap();
    assert!(manager.contains("people.txt > id:entier|name:chaine|age:entier"));

    let renamed_again = run(&root, "afficher tous les 'people'");
    assert!(renamed_again.success, "{:?}", renamed_again.message);

    let dropped = run(&root, "supprimer fichier 'people'");
    assert!(dropped.success, "{:?}", dropped.message);
    assert!(!root.join("people.txt").exists());
    assert!(!fs::read_to_string(root.join("manager.txt")).unwrap().contains("people"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn runs_scripts_with_comments_and_stops_on_failure() {
    let root = workspace();

    let script = "# setup\
\ncreer fichier 'users' ( $id : entier , $name : chaine )\
\ninserer dans 'users' : $name = 'aa'\
\nafficher tous les 'users'\
\n";
    let results = run_script(&root, script);
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|result| result.success));

    let second = run(&root, "afficher tous les 'users' si $id = 1");
    assert_eq!(second.row_count, Some(1));

    let failing = "afficher tous les 'users'\neffacer de 'users' si $id 123\nafficher tous les 'users'\n";
    let results = run_script(&root, failing);
    assert_eq!(results.len(), 2);
    assert!(results[0].success);
    assert!(!results[1].success);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn keeps_wrapped_creatable_statement_whole() {
    let root = workspace();

    // The parens keep the wrapped statement together despite the newline.
    let script = "creer fichier 'users' (\
\n$id : entier ,\
\n$name : chaine\
\n)\
\ninserer dans 'users' : $name = 'aa'\
\n";
    let results = run_script(&root, script);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|result| result.success));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reserves_critical_table_names() {
    let root = workspace();

    let manager = run(&root, "creer fichier 'manager' ( $id : entier )");
    assert!(!manager.success);
    assert!(manager.message.unwrap().contains("réservé"));

    // The workspace still works afterwards.
    let created = run(&root, "creer fichier 'users' ( $id : entier )");
    assert!(created.success, "{:?}", created.message);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_issues_are_reported() {
    let root = workspace();
    fs::write(
        root.join("manager.txt"),
        "users.txt > id:entier|name:chaine\nnot a catalog line\nghost.txt > id:entier\n",
    )
    .unwrap();
    fs::write(root.join("users.txt"), "1|aa\n").unwrap();
    fs::write(root.join("orphan.txt"), "x|y\n").unwrap();

    let issues = workspace_issues(&root).unwrap();
    assert!(issues.iter().any(|issue| issue.contains("format invalide")));
    assert!(issues.iter().any(|issue| issue.contains("orphelin") && issue.contains("orphan.txt")));
    assert!(issues.iter().any(|issue| issue.contains("ghost") && issue.contains("absent")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn supports_new_types() {
    let root = workspace();

    let created = run(
        &root,
        "creer fichier 'produits' ( $id : entier , $prix : reel , $actif : booleen , $initiale : caractere , $nom : chaine )",
    );
    assert!(created.success, "{:?}", created.message);

    let inserted = run(
        &root,
        "inserer dans 'produits' : $prix = 10.5 , $actif = vrai , $initiale = 'A' , $nom = 'stylo'",
    );
    assert!(inserted.success, "{:?}", inserted.message);

    let wrong_reel = run(
        &root,
        "inserer dans 'produits' : $prix = '10.5' , $actif = vrai , $initiale = 'A' , $nom = 'x'",
    );
    assert!(!wrong_reel.success);
    assert!(wrong_reel.message.unwrap().contains("doit être de type reel"));

    let wrong_char = run(
        &root,
        "inserer dans 'produits' : $prix = 1 , $actif = vrai , $initiale = 'AB' , $nom = 'x'",
    );
    assert!(!wrong_char.success);
    assert!(wrong_char.message.unwrap().contains("doit être de type caractere"));

    let wrong_bool = run(
        &root,
        "inserer dans 'produits' : $prix = 1 , $actif = 'vrai' , $initiale = 'A' , $nom = 'x'",
    );
    assert!(!wrong_bool.success);
    assert!(wrong_bool.message.unwrap().contains("doit être de type booleen"));

    let zero_rows = run(&root, "afficher tous les 'produits' si $actif = faux");
    assert_eq!(zero_rows.row_count, Some(0));
    let one_row = run(&root, "afficher tous les 'produits' si $actif = vrai");
    assert_eq!(one_row.row_count, Some(1));

    let by_price = run(&root, "afficher tous les 'produits' si $prix > 10");
    assert_eq!(by_price.row_count, Some(1));

    // Booleans keep a canonical lowercase form on disk.
    let file = fs::read_to_string(root.join("produits.txt")).unwrap();
    assert!(file.contains("vrai"));
    assert!(!file.contains("VRAI"));

    let updated = run(&root, "modifier 'produits' si $id = 1 : $prix = 9.99 , $actif = faux");
    assert!(updated.success, "{:?}", updated.message);

    let displayed = run(&root, "afficher tous les 'produits' si $prix = 9.99");
    assert_eq!(displayed.row_count, Some(1));
    let rows = displayed.rows.as_ref().unwrap();
    assert_eq!(rows[0][1], "9.99");
    assert_eq!(rows[0][2], "faux");

    // After the change, the old boolean value no longer matches.
    assert_eq!(run(&root, "afficher tous les 'produits' si $actif = vrai").row_count, Some(0));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stores_special_characters_safely() {
    let root = workspace();

    let created = run(&root, "creer fichier 'notes' ( $id : entier , $texte : chaine )");
    assert!(created.success, "{:?}", created.message);

    // Pipes, comparison symbols, backslashes, lone quotes and newlines must
    // all survive a write/read cycle.
    let cases: Vec<(&str, &str)> = vec![
        ("inserer dans 'notes' : $texte = 'a|b'", "a|b"),
        ("inserer dans 'notes' : $texte = 'a > b'", "a > b"),
        ("inserer dans 'notes' : $texte = 'a\\b'", "a\\b"),
        ("inserer dans 'notes' : $texte = ''''", "'"),
        ("inserer dans 'notes' : $texte = 'it''s'", "it's"),
        (
            "inserer dans 'notes' : $texte = 'ligne1\nligne2'",
            "ligne1\nligne2",
        ),
    ];
    for (query, expected) in &cases {
        let inserted = run(&root, query);
        assert!(inserted.success, "{query} -> {:?}", inserted.message);
        let displayed = run(&root, "afficher tous les 'notes'");
        let rows = displayed.rows.as_ref().unwrap();
        assert_eq!(&rows.last().unwrap()[1], expected, "{query}");
    }

    // The file on disk keeps the v3 header and escaped encoding.
    let file = fs::read_to_string(root.join("notes.txt")).unwrap();
    assert!(file.starts_with("# fakesql format 3\n"));
    assert!(file.contains("a\\|b"));
    assert!(file.contains("ligne1\\nligne2"));

    // Conditions can target the special values.
    let filtered = run(&root, "afficher tous les 'notes' si $texte = 'a|b'");
    assert_eq!(filtered.row_count, Some(1));

    let removed = run(&root, "effacer de 'notes' si $texte = 'a|b'");
    assert_eq!(removed.affected, Some(1));
    assert_eq!(run(&root, "afficher tous les 'notes'").row_count, Some(5));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn keeps_legacy_tables_readable_and_upgrades_on_write() {
    let root = workspace();
    fs::write(root.join("manager.txt"), "users.txt > id:entier|texte:chaine\n").unwrap();
    fs::write(root.join("users.txt"), "1|valeur une\n2|valeur deux\n").unwrap();

    let displayed = run(&root, "afficher tous les 'users'");
    assert!(displayed.success, "{:?}", displayed.message);
    assert_eq!(displayed.rows.as_ref().unwrap().len(), 2);

    let inserted = run(&root, "inserer dans 'users' : $texte = 'trois|avec tube'");
    assert!(inserted.success, "{:?}", inserted.message);

    // The rewritten file starts with the v3 header; earlier rows survive.
    let file = fs::read_to_string(root.join("users.txt")).unwrap();
    assert!(file.starts_with("# fakesql format 3\n"));

    let displayed = run(&root, "afficher tous les 'users'");
    let rows = displayed.rows.as_ref().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0], vec!["1", "valeur une"]);
    assert_eq!(rows[2], vec!["3", "trois|avec tube"]);

    fs::remove_dir_all(root).unwrap();
}
