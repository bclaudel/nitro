mod cli;
mod connect;
mod list;
mod shell;
mod tmux;
mod zoxide;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use connect::ConnectOptions;
use list::ListOptions;
use shell::RealShell;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = RealShell;

    match cli.command {
        Commands::List {
            tmux,
            zoxide,
            icons,
            no_color,
        } => {
            let opts = list_options_from_flags(tmux, zoxide, icons, no_color);
            list::run_list(&sh, opts)
        }
        Commands::Connect {
            dir,
            no_color: _,
            no_fail,
            name_tokens,
        } => {
            let opts = ConnectOptions {
                tokens: name_tokens,
                dir,
            };
            match connect::run_connect(&sh, opts) {
                Ok(()) => Ok(()),
                Err(e) if no_fail => {
                    eprintln!("{e}");
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }
}

fn list_options_from_flags(
    tmux: bool,
    zoxide: Option<Option<usize>>,
    icons: bool,
    no_color: bool,
) -> ListOptions {
    let include_tmux = if tmux { true } else { zoxide.is_none() };
    let (include_zox, z_limit) = match zoxide {
        None => (!tmux, None),
        Some(inner) => (true, inner),
    };
    ListOptions {
        include_tmux,
        include_zox,
        z_limit,
        icons,
        no_color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_flag_translation_defaults() {
        // No flags => include_tmux true, include_zox true, no limit
        let lo = list_options_from_flags(false, None, false, true);
        assert!(lo.include_tmux);
        assert!(lo.include_zox);
        assert_eq!(lo.z_limit, None);
        assert!(!lo.icons);
        assert!(lo.no_color);
    }

    #[test]
    fn list_flag_translation_tmux_only() {
        let lo = list_options_from_flags(true, None, false, true);
        assert!(lo.include_tmux);
        assert!(!lo.include_zox);
    }

    #[test]
    fn list_flag_translation_zoxide_only_unlimited() {
        let lo = list_options_from_flags(false, Some(None), false, false);
        assert!(!lo.include_tmux);
        assert!(lo.include_zox);
        assert_eq!(lo.z_limit, None);
    }

    #[test]
    fn list_flag_translation_zoxide_with_limit() {
        let lo = list_options_from_flags(false, Some(Some(5)), true, false);
        assert!(!lo.include_tmux);
        assert!(lo.include_zox);
        assert_eq!(lo.z_limit, Some(5));
        assert!(lo.icons);
    }
}
