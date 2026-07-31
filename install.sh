#!/usr/bin/env sh
set -eu

REPO="${JCA_TMUXER_REPO:-sisocobacho/jca_tmuxer}"
BINARY_NAME="jca_tmuxer"
COMPLETION_BINARY_NAME="jca_tmuxer-completions"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"
INSTALL_COMPLETIONS="${INSTALL_COMPLETIONS:-0}"
COMPLETION_SHELL="${COMPLETION_SHELL:-}"
COMPLETION_BINS="${COMPLETION_BINS:-jca_tmuxer jtmx}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'error: required command not found: %s\n' "$1" >&2
    exit 1
  fi
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  if [ "$os" != "Linux" ]; then
    printf 'error: unsupported OS: %s (Linux only installer)\n' "$os" >&2
    exit 1
  fi

  case "$arch" in
    x86_64 | amd64)
      printf 'x86_64-unknown-linux-musl'
      ;;
    aarch64 | arm64)
      printf 'aarch64-unknown-linux-musl'
      ;;
    *)
      printf 'error: unsupported architecture: %s\n' "$arch" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [ "$VERSION" = "latest" ]; then
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
      awk 'NR==1 {print; exit}'
  else
    printf '%s' "$VERSION"
  fi
}

verify_checksum() {
  archive_path="$1"
  checksums="$2"
  archive_name="$(basename "$archive_path")"
  expected="$(grep "  ${archive_name}$" "$checksums" | awk '{print $1}')"
  if [ -z "$expected" ]; then
    printf 'error: missing checksum entry for %s\n' "$archive_name" >&2
    exit 1
  fi

  actual="$(sha256sum "$archive_path" | awk '{print $1}')"
  if [ "$actual" != "$expected" ]; then
    printf 'error: checksum verification failed for %s\n' "$archive_name" >&2
    exit 1
  fi
}

detect_completion_shell() {
  if [ -n "$COMPLETION_SHELL" ]; then
    case "$COMPLETION_SHELL" in
      bash | zsh | fish)
        printf '%s' "$COMPLETION_SHELL"
        return 0
        ;;
      *)
        return 1
        ;;
    esac
  fi

  shell_path="${SHELL:-}"
  if [ -z "$shell_path" ]; then
    return 1
  fi

  shell_name="$(basename "$shell_path")"
  case "$shell_name" in
    bash | zsh | fish)
      printf '%s' "$shell_name"
      ;;
    *)
      return 1
      ;;
  esac
}

completion_target_path() {
  shell_name="$1"
  bin_name="$2"

  case "$shell_name" in
    bash)
      printf '%s/.local/share/bash-completion/completions/%s' "$HOME" "$bin_name"
      ;;
    zsh)
      printf '%s/.zfunc/_%s' "$HOME" "$bin_name"
      ;;
    fish)
      printf '%s/.config/fish/completions/%s.fish' "$HOME" "$bin_name"
      ;;
    *)
      return 1
      ;;
  esac
}

install_completions() {
  if [ "$INSTALL_COMPLETIONS" != "1" ]; then
    return 0
  fi

  helper_path="$INSTALL_DIR/$COMPLETION_BINARY_NAME"
  if [ ! -x "$helper_path" ]; then
    printf 'warning: %s not found at %s; skipping completion install\n' "$COMPLETION_BINARY_NAME" "$helper_path" >&2
    return 0
  fi

  shell_name="$(detect_completion_shell || true)"
  if [ -z "$shell_name" ]; then
    printf 'warning: unable to detect supported shell (bash, zsh, fish); set COMPLETION_SHELL to override\n' >&2
    return 0
  fi

  for bin_name in $COMPLETION_BINS; do
    target_path="$(completion_target_path "$shell_name" "$bin_name")"
    target_dir="$(dirname "$target_path")"
    mkdir -p "$target_dir"
    if "$helper_path" "$shell_name" --bin-name "$bin_name" > "$target_path"; then
      printf 'Installed %s completion for %s at %s\n' "$shell_name" "$bin_name" "$target_path"
    else
      printf 'warning: failed to install %s completion for %s\n' "$shell_name" "$bin_name" >&2
    fi
  done

  if [ "$shell_name" = "zsh" ]; then
    printf 'zsh hint: ensure ~/.zfunc is in fpath, then run: autoload -Uz compinit && compinit\n'
  fi
}

main() {
  need_cmd uname
  need_cmd curl
  need_cmd tar
  need_cmd mktemp
  need_cmd sha256sum
  need_cmd grep
  need_cmd awk
  need_cmd sed
  need_cmd basename
  need_cmd dirname
  need_cmd install

  target="$(detect_target)"
  version_tag="$(resolve_version)"
  if [ -z "$version_tag" ]; then
    printf 'error: failed to determine release version\n' >&2
    exit 1
  fi

  archive="${BINARY_NAME}-${target}.tar.gz"
  base_url="https://github.com/${REPO}/releases/download/${version_tag}"

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT INT TERM

  printf 'Installing %s %s for %s\n' "$BINARY_NAME" "$version_tag" "$target"
  curl -fsSL "$base_url/$archive" -o "$tmpdir/$archive"
  curl -fsSL "$base_url/checksums.txt" -o "$tmpdir/checksums.txt"

  verify_checksum "$tmpdir/$archive" "$tmpdir/checksums.txt"

  mkdir -p "$INSTALL_DIR"
  tar -xzf "$tmpdir/$archive" -C "$tmpdir"
  install "$tmpdir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
  if [ -f "$tmpdir/$COMPLETION_BINARY_NAME" ]; then
    install "$tmpdir/$COMPLETION_BINARY_NAME" "$INSTALL_DIR/$COMPLETION_BINARY_NAME"
  fi

  printf 'Installed to %s/%s\n' "$INSTALL_DIR" "$BINARY_NAME"
  if [ -f "$INSTALL_DIR/$COMPLETION_BINARY_NAME" ]; then
    printf 'Installed to %s/%s\n' "$INSTALL_DIR" "$COMPLETION_BINARY_NAME"
  fi
  case ":$PATH:" in
    *":$INSTALL_DIR:"*)
      ;;
    *)
      printf 'warning: %s is not in PATH\n' "$INSTALL_DIR" >&2
      ;;
  esac

  install_completions
}

main "$@"
