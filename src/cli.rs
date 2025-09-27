use clap::{ArgAction, Parser, Subcommand, ValueHint};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "nitro", version, about = "Fast tmux sessions via zoxide")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List tmux sessions and/or zoxide results
    List {
        /// Include tmux sessions
        #[arg(short = 't', long = "tmux", action = ArgAction::SetTrue)]
        tmux: bool,
        /// Include zoxide results; optional N limits to top N (e.g., -z 10)
        /// Presence without value means all results
        #[arg(short = 'z', long = "zoxide", num_args = 0..=1, value_parser = clap::value_parser!(usize))]
        zoxide: Option<Option<usize>>,
        /// Use fancy icons (  for tmux,   for zoxide) instead of ASCII [t]/[z]
        #[arg(long = "icons", action = ArgAction::SetTrue)]
        icons: bool,
        /// Disable colored output (overrides NO_COLOR)
        #[arg(long = "no-color", action = ArgAction::SetTrue)]
        no_color: bool,
    },

    /// Connect to a tmux session, creating it if missing
    Connect {
        /// Optional working directory override for new sessions
        #[arg(long = "dir", value_hint = ValueHint::DirPath)]
        dir: Option<PathBuf>,
        /// Disable colored output (overrides NO_COLOR)
        #[arg(long = "no-color", action = ArgAction::SetTrue)]
        no_color: bool,
        /// Do not fail: exit 0 even if an error occurs
        #[arg(long = "no-fail", action = ArgAction::SetTrue)]
        no_fail: bool,
        /// Full line tokens (supports fzf piping without quotes)
        #[arg(name = "name", trailing_var_arg = true, num_args = 1..)]
        name_tokens: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list_flags_variants() {
        // Default: no flags
        let c = Cli::parse_from(["nitro", "list"]);
        match c.command {
            Commands::List {
                tmux,
                zoxide,
                icons,
                no_color,
            } => {
                assert!(!tmux);
                assert!(zoxide.is_none());
                assert!(!icons);
                assert!(!no_color);
            }
            _ => panic!("expected list"),
        }

        // tmux only
        let c = Cli::parse_from(["nitro", "list", "--tmux"]);
        match c.command {
            Commands::List { tmux, .. } => assert!(tmux),
            _ => panic!(),
        }

        // zoxide with count
        let c = Cli::parse_from(["nitro", "list", "-z", "10"]);
        match c.command {
            Commands::List { zoxide, .. } => assert_eq!(zoxide, Some(Some(10))),
            _ => panic!(),
        }

        // zoxide presence without value => Some(None)
        let c = Cli::parse_from(["nitro", "list", "-z"]);
        match c.command {
            Commands::List { zoxide, .. } => assert_eq!(zoxide, Some(None)),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_connect_args_dir_and_name() {
        // dir flag
        let c = Cli::parse_from(["nitro", "connect", "--dir", "/tmp", "web", "prod"]);
        match c.command {
            Commands::Connect {
                dir,
                name_tokens,
                no_color,
                no_fail,
            } => {
                assert_eq!(dir.as_deref(), Some(std::path::Path::new("/tmp")));
                assert_eq!(name_tokens, vec!["web".to_string(), "prod".to_string()]);
                assert!(!no_color);
                assert!(!no_fail);
            }
            _ => panic!("expected connect"),
        }

        // no-color flag
        let c = Cli::parse_from(["nitro", "connect", "--no-color", "api"]);
        match c.command {
            Commands::Connect {
                no_color,
                name_tokens,
                no_fail,
                ..
            } => {
                assert!(no_color);
                assert!(!no_fail);
                assert_eq!(name_tokens, vec!["api".to_string()]);
            }
            _ => panic!(),
        }

        // no-fail flag
        let c = Cli::parse_from(["nitro", "connect", "--no-fail", "api"]);
        match c.command {
            Commands::Connect { no_fail, .. } => {
                assert!(no_fail);
            }
            _ => panic!(),
        }
    }
}
