**Build/Test**
- Build: `cargo build` (Rust 2024 edition).
- Run: `cargo run -- <subcommand>` (e.g., `list`, `connect`).
- Test all: `cargo test`.
- Test single: `cargo test list_tmux_and_zoxide_ascii_and_limit` (substring match works).
- Lint: `cargo clippy --all-targets -- -D warnings`.
- Format check: `cargo fmt -- --check`; auto-fix with `cargo fmt`.

**Repo Conventions**
- Language: Rust, single binary crate; unit tests colocated in modules.
- Imports: group `std`, external crates, then `crate::…`; avoid wildcard `use *`.
- Types: prefer `anyhow::Result<T>`; use `usize` for counts, `PathBuf` for paths.
- Errors: add context with `with_context`; use `bail!`/`ensure!`; for non-critical shell listings it’s OK to degrade gracefully (see `unwrap_or_default` usage).
- Naming: snake_case for functions/vars, CamelCase for types, SCREAMING_SNAKE_CASE for consts.
- CLI: use `clap` derives; flags are kebab-case; preserve semantics in `cli.rs` (`--no-color`, `--icons`, `--zoxide [N]`).
- Color: respect `NO_COLOR` and `--no-color`; keep ASCII vs icon modes consistent with `list.rs`.
- External tools: relies on `tmux` and `zoxide`; abstract calls via `Shell` trait (`shell.rs`).
- Testing: follow `list.rs` pattern with a `MockShell` to isolate command effects.
- Ordering: sort user-facing lists for stable output (see `tmux::list_sessions`).
- Cursor/Copilot: no `.cursor/rules/`, `.cursorrules`, or Copilot instructions found.
