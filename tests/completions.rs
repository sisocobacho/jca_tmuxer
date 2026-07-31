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
fn completion_generator_rejects_invalid_shell() {
    let mut cmd = Command::cargo_bin("jca_tmuxer-completions").expect("bin");
    cmd.arg("invalid-shell")
        .assert()
        .failure()
        .stderr(contains("invalid value"));
}
