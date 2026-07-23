use jca_tmuxer::plan::{PanePlan, WindowPlan};
use jca_tmuxer::tmux::dry_run_commands;
use std::path::PathBuf;

#[test]
fn builds_dry_run_commands() {
    let windows = vec![WindowPlan {
        name: "editor".to_string(),
        layout: "stacked".to_string(),
        panes: vec![PanePlan {
            command: "nvim".to_string(),
            cwd: PathBuf::from("/tmp/app"),
            size: None,
        }],
    }];

    let cmds = dry_run_commands("app", &windows, false);
    assert!(cmds.iter().any(|c| c.contains("new-session")));
    assert!(cmds.iter().any(|c| c.contains("attach-session")));
}
