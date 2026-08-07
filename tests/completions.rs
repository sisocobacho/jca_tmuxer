use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn completion_generator_supports_all_shells() {
    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
        let output = cmd
            .arg(shell)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let text = String::from_utf8(output).expect("utf8 output");
        assert!(
            text.contains("dry-run"),
            "expected dry-run in {shell} output"
        );
        assert!(text.contains("config"), "expected config in {shell} output");
    }
}

#[test]
fn completion_generator_uses_custom_bin_name() {
    let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
    cmd.arg("bash")
        .arg("--bin-name")
        .arg("jtmx")
        .assert()
        .success()
        .stdout(contains("jtmx"));
}

#[test]
fn bash_completion_includes_dynamic_project_lookup() {
    let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
    let output = cmd
        .arg("bash")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        text.contains("__jca_tmuxer_dynamic_projects"),
        "expected bash dynamic completion hook"
    );
    assert!(
        text.contains("jca_tmuxer --list"),
        "expected bash completion to call --list"
    );
    assert!(
        text.contains("_jca_tmuxer \"$1\" \"$cur\" \"$prev\""),
        "expected bash completion to pass expected fallback args"
    );
    assert!(
        text.contains("complete -o nosort -o bashdefault -o default -F __jca_tmuxer_dynamic_completion jca_tmuxer"),
        "expected bash completion to preserve bashdefault/default options"
    );
}

#[test]
fn zsh_completion_includes_dynamic_project_lookup() {
    let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
    let output = cmd
        .arg("zsh")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        text.contains("_jca_tmuxer_dynamic_projects"),
        "expected zsh dynamic completion hook"
    );
    assert!(
        text.contains("jca_tmuxer --list"),
        "expected zsh completion to call --list"
    );
}

#[test]
fn dynamic_completion_uses_custom_bin_name() {
    let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
    let output = cmd
        .arg("zsh")
        .arg("--bin-name")
        .arg("jtmx")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        text.contains("jtmx --list"),
        "expected dynamic completion hook to use custom bin name"
    );
}

#[test]
fn completion_generator_rejects_invalid_shell() {
    let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
    cmd.arg("invalid-shell")
        .assert()
        .failure()
        .stderr(contains("invalid value"));
}
