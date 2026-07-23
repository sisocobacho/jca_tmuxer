# jca_tmuxer

`jca_tmuxer` is a project-aware tmux launcher that creates or reuses a tmux session per project, opens predefined windows/panes, and optionally appends ad-hoc commands.

## Features

- Launch tmux sessions by project name or path
- Reuse existing sessions automatically
- Define default and project-specific windows in YAML
- Set per-window and per-pane working directories
- Add one-off ad-hoc commands at launch
- Dry-run mode to preview tmux commands before execution

## Requirements

- `tmux` installed and available in `PATH`
- Rust toolchain (`cargo`) for building/installing

Check tmux:

```bash
tmux -V
```

## Installation

### From source (current repo)

```bash
cargo install --path .
```

### Development run

```bash
cargo run -- <project_name>
```

## Quick Start

1. Create config directory:

```bash
mkdir -p ~/.config/jca_tmuxer
```

2. Create `~/.config/jca_tmuxer/config.yaml`:

```yaml
defaults:
  layout: stacked
  windows:
    - name: editor
      command: nvim
      directory: "<project_root>"
    - name: git
      command: lazygit
      directory: "<project_root>"
    - name: terminal
      command: bash
      directory: "<project_root>"

projects:
  my_project:
    root: ~/code/my_project
    windows:
      - name: editor
        command: nvim
      - name: api
        command: npm run dev
        directory: ~/code/my_project/packages/api
      - name: frontend
        command: npm start
        directory: ~/code/my_project/packages/web
```

3. Launch:

```bash
jca_tmuxer my_project
```

If session `my_project` exists, `jca_tmuxer` attaches to it. If not, it creates it and runs configured windows/commands.

## Usage

```bash
jca_tmuxer <PROJECT> [ADHOC_COMMANDS]... [OPTIONS]
```

### Basic

```bash
jca_tmuxer my_project
```

### Add ad-hoc commands

```bash
jca_tmuxer my_project "npm run dev" "npm test"
```

Each ad-hoc command opens in a new window in project root.

### Ad-hoc command with custom directory

```bash
jca_tmuxer my_project "~/code/my_project/packages/api:npm run dev"
```

### Force fresh session

```bash
jca_tmuxer my_project --new
```

### Preview without executing

```bash
jca_tmuxer my_project --dry-run
```

### List configured projects

```bash
jca_tmuxer --list
```

### Print resolved window plan

```bash
jca_tmuxer my_project --print-config
```

## Ad-hoc Command Syntax

- `"command"` runs in project root
- `"/path/to/dir:command"` runs in specified directory
- escape literal colon with `\:`

Examples:

```bash
jca_tmuxer my_project "echo hello"
jca_tmuxer my_project "/tmp:ls -la"
jca_tmuxer my_project "my\:label:echo ok"
```

## Configuration

### File locations and precedence

Highest to lowest priority:

1. `--config <path>`
2. `JCA_TMUXER_CONFIG` environment variable
3. `~/.config/jca_tmuxer/config.yaml`
4. Local `.jca_tmuxer.yaml` found by walking up from current directory (merged as override)
5. Built-in defaults

### Supported keys

- `search_paths`: fallback directories for project discovery
- `defaults.layout`: default window layout
- `defaults.windows`: default windows list
- `projects.<name>.root`: project root path
- `projects.<name>.extend`: merge with defaults when true
- `projects.<name>.windows`: project window definitions

Window fields:

- `name`
- `command`
- `directory`
- `layout`
- `panes` (list of pane objects)

Pane fields:

- `command`
- `directory`
- `size` (percentage)

See full example: `examples/config.yaml`.

## Session Behavior

- Session name is project name (or basename when project is provided as a path)
- If session exists, default behavior is attach/switch
- If ad-hoc commands are provided and session exists, ad-hoc windows are appended
- `--new` kills existing session and recreates it

## Troubleshooting

### `tmux is not installed or not in PATH`

Install tmux and verify:

```bash
tmux -V
```

### `unknown project '<name>'`

- Add project under `projects:` in config, or
- Pass an absolute/relative project path directly

### `directory does not exist`

One of the configured `directory` values is invalid. Check project root and window/pane paths.

### Not attaching in scripts/CI

If stdin is not a TTY, attach is skipped automatically. Use `--no-attach` for non-interactive workflows.

## Development

Run checks:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Changelog

See `CHANGELOG.md`.
