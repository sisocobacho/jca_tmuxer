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

#[test]
fn dry_run_includes_kill_session_when_force_new() {
    let windows = vec![WindowPlan {
        name: "editor".to_string(),
        layout: "stacked".to_string(),
        panes: vec![PanePlan {
            command: "nvim".to_string(),
            cwd: PathBuf::from("/tmp/app"),
            size: None,
        }],
    }];

    let cmds = dry_run_commands("app", &windows, true);
    assert!(
        cmds.iter().any(|c| c.contains("kill-session -t app")),
        "expected kill-session command in: {cmds:?}"
    );
}

#[test]
fn dry_run_covers_multi_window_and_pane_flow() {
    let windows = vec![
        WindowPlan {
            name: "editor".to_string(),
            layout: "stacked".to_string(),
            panes: vec![
                PanePlan {
                    command: "nvim".to_string(),
                    cwd: PathBuf::from("/tmp/app"),
                    size: None,
                },
                PanePlan {
                    command: "cargo test".to_string(),
                    cwd: PathBuf::from("/tmp/app"),
                    size: Some(40),
                },
            ],
        },
        WindowPlan {
            name: "api".to_string(),
            layout: "tiled".to_string(),
            panes: vec![PanePlan {
                command: "npm run dev".to_string(),
                cwd: PathBuf::from("/tmp/app/api"),
                size: None,
            }],
        },
    ];

    let cmds = dry_run_commands("app", &windows, false);

    assert_eq!(
        cmds.first().expect("first command"),
        "tmux has-session -t app"
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("new-session -d -s app -n editor"))
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("split-window -t app:editor -c /tmp/app"))
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("send-keys -t app:editor.1 'cargo test' C-m"))
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("resize-pane -t app:editor.1 -p 40"))
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("new-window -t app -n api -c /tmp/app/api"))
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("select-layout -t app:editor stacked"))
    );
    assert!(
        cmds.iter()
            .any(|c| c.contains("select-layout -t app:api tiled"))
    );
    assert_eq!(
        cmds.last().expect("last command"),
        "tmux attach-session -t app"
    );
}
