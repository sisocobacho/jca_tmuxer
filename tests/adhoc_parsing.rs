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
