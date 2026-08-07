# Changelog

## 0.5.0

- Add dynamic project-name autocomplete for `bash` and `zsh` completion scripts
- Source completion candidates from configured projects via `jca_tmuxer --list`
- Preserve bash completion fallback behavior with `bashdefault`/`default` options
- Add completion tests for dynamic hooks and custom `--bin-name` integration
- Document shell completion support matrix and shell loading/troubleshooting steps

## 0.4.0

- Add `jca_tmuxer-completions` helper binary to generate shell completion scripts
- Add completion generation tests across supported shells and custom bin names
- Add path hints in CLI args to improve shell completion quality for `--config` and `--root`
- Add opt-in completion install in Linux installer via `INSTALL_COMPLETIONS=1`
- Add installer support for `COMPLETION_SHELL` and `COMPLETION_BINS`
- Package `jca_tmuxer-completions` in GitHub release archives
- Document manual and installer-based completion setup for `jca_tmuxer` and `jtmx`

## 0.3.0

- Add `--remove` mode to delete a project from config and remove its tmux session
- Add interactive removal confirmation prompt with `--yes` override for non-interactive usage
- Print `Nothing was removed.` when no matching config project or tmux session exists
- Print concise removal results for config project and tmux session when removed
- Expand integration and behavior tests for remove flow and idempotent semantics

## 0.2.0

- Add GitHub Release workflow for Linux musl binaries (`x86_64` and `aarch64`)
- Publish release artifacts with `sha256` checksums
- Add one-command Linux installer script (`install.sh`) with checksum verification
- Document binary installation and manual download flow in README

## 0.1.0

- Initial release of `jca_tmuxer`
- YAML-based project/default window configuration
- Ad-hoc command windows with optional `dir:command` syntax
- Session detection, optional `--new`, attach/switch behavior
- Dry-run command rendering and integration tests
