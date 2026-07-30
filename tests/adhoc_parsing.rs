use jca_tmuxer::adhoc::parse_adhoc;
use std::path::Path;

#[test]
fn parses_plain_command_to_project_root() {
    let parsed = parse_adhoc(&["npm run dev".to_string()], Path::new("/tmp/project"));
    assert_eq!(parsed[0].command, "npm run dev");
    assert_eq!(parsed[0].directory, Path::new("/tmp/project"));
}

#[test]
fn parses_directory_prefix() {
    let parsed = parse_adhoc(
        &["/tmp/project/api:npm run dev".to_string()],
        Path::new("/tmp/project"),
    );
    assert_eq!(parsed[0].command, "npm run dev");
    assert_eq!(parsed[0].directory, Path::new("/tmp/project/api"));
}

#[test]
fn preserves_escaped_colon_in_command() {
    let parsed = parse_adhoc(
        &["my\\:label:echo ok".to_string()],
        Path::new("/tmp/project"),
    );
    assert_eq!(parsed[0].command, "my:label:echo ok");
    assert_eq!(parsed[0].directory, Path::new("/tmp/project"));
}

#[test]
fn falls_back_to_project_root_for_non_path_prefix_before_colon() {
    let parsed = parse_adhoc(
        &["label:npm run dev".to_string()],
        Path::new("/tmp/project"),
    );
    assert_eq!(parsed[0].command, "label:npm run dev");
    assert_eq!(parsed[0].directory, Path::new("/tmp/project"));
}
