# Nitro

[![CI](https://github.com/bclaudel/nitro/actions/workflows/ci.yml/badge.svg)](https://github.com/bclaudel/nitro/actions/workflows/ci.yml)

Fast tmux sessions via zoxide.

## Install

- Requires `tmux` and `zoxide` on PATH.
- Build from source:

```
cargo install --path .
```

## Usage

```
nitro list [-t] [-z[ N]] [--icons] [--no-color]
nitro connect <name...> [--dir PATH] [--no-color]
```

- `nitro list` defaults to both sources (`-t -z`).
- `-t/--tmux`: include tmux sessions.
- `-z/--zoxide [N]`: include zoxide results; optional `N` limits to top N. Without a value, prints all.
- Default prefixes: `[t]` for tmux and `[z]` for zoxide.
- Colors:
  - ASCII mode: `[t]` is magenta, `[z]` is blue.
  - `--icons` mode: `` (magenta) for tmux, `` (blue) for zoxide with two spaces after the icon for clarity.
  - `--no-color` disables colors regardless of the `NO_COLOR` env.

Zoxide lines print as absolute `path` only (no basename).

## Examples

- Combined (default ASCII):
  - `nitro list`
- Combined with icons:
  - `nitro list --icons`
- Only tmux:
  - `nitro list -t`
- Top 5 zoxide with icons:
  - `nitro list -z 5 --icons`
- No colors:
  - `nitro list --no-color`
- Chain with fzf (works in both ASCII and icon modes):
  - `nitro connect $(nitro list --icons | fzf)`

## Behavior

- `connect` strips icons (`[t]`, `[z]`) and accepts unquoted lines from fzf.
- Path detection: first absolute-token starting with `/` marks path; tokens before it form the session name.
- New session directory: `--dir` > parsed path > `zoxide query <name>` best match > `$HOME`.
- Inside tmux: uses `tmux switch-client`; otherwise `tmux attach`.

## Contributing

- Enable git hooks: the repo uses a pre-commit hook in `.githooks/pre-commit`.
  - Ensure hooks path is configured (done automatically in this repo): `git config core.hooksPath .githooks`
- Lint and format locally before committing:
  - Lint: `cargo lint` (alias for `clippy --all-targets -- -D warnings`)
  - Format check: `cargo fmt -- --check` (auto-fix with `cargo fmt`)
- CI runs clippy, fmt check, and tests on pushes/PRs.

