use std::path::PathBuf;

use clap::{Parser, Subcommand};
#[cfg(feature = "interactive")]
use skillinstaller::install_interactive;
use skillinstaller::{
    detect_providers, print_install_result, supported_providers, InstallSkillArgs, SkillSource,
};
#[cfg(not(feature = "interactive"))]
use skillinstaller::{install, parse_providers_csv, InstallRequest};

#[derive(Debug, Parser)]
#[command(name = "install-skill")]
#[command(about = "Developer tooling for installing .skill payloads across providers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List supported providers
    Providers,

    /// Detect installed providers on this machine
    Detect {
        /// Project root used for project-level detection hints
        #[arg(long)]
        project_root: Option<PathBuf>,
    },

    /// Install a .skill payload
    Install {
        /// Path containing .skill/ (or a direct .skill path)
        #[arg(long)]
        source: Option<PathBuf>,

        #[command(flatten)]
        args: InstallSkillArgs,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Providers => cmd_providers(),
        Commands::Detect { project_root } => cmd_detect(project_root),
        Commands::Install { source, args } => cmd_install(source, args),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn cmd_providers() -> Result<(), String> {
    for p in supported_providers() {
        let mode = if p.uses_agents_dir {
            "shared .agents"
        } else {
            "provider-specific"
        };
        println!("{}\t{}\t{}", p.id.as_str(), p.display_name, mode);
    }
    Ok(())
}

fn cmd_detect(project_root: Option<PathBuf>) -> Result<(), String> {
    let detected = detect_providers(project_root.as_deref());
    if detected.is_empty() {
        println!("no providers detected");
        return Ok(());
    }

    for d in detected {
        println!("{}\t{}", d.provider.as_str(), d.reason);
    }

    Ok(())
}

fn cmd_install(source: Option<PathBuf>, args: InstallSkillArgs) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("failed to read cwd: {e}"))?;
    let source = SkillSource::LocalPath(source.unwrap_or(cwd));

    #[cfg(feature = "interactive")]
    {
        let result = install_interactive(source, &args).map_err(|e| e.to_string())?;
        print_install_result(&result);
        return Ok(());
    }

    #[cfg(not(feature = "interactive"))]
    {
        let all_specified =
            args.providers.is_some() && args.scope.is_some() && args.method.is_some();
        if !all_specified {
            return Err(
                "interactive mode requires 'interactive' feature; provide --providers, --scope, and --method"
                    .to_string(),
            );
        }

        let providers =
            parse_providers_csv(args.providers.as_deref().unwrap()).map_err(|e| e.to_string())?;
        let scope = args.scope.unwrap();
        let method = args.method.unwrap();
        let project_root = match scope {
            skillinstaller::Scope::User => None,
            skillinstaller::Scope::Project => {
                Some(args.project_root.unwrap_or_else(|| match &source {
                    SkillSource::LocalPath(p) => p.clone(),
                    _ => std::path::PathBuf::from("."),
                }))
            }
        };

        let result = install(InstallRequest {
            source,
            providers,
            scope,
            project_root,
            method,
            force: args.force,
        })
        .map_err(|e| e.to_string())?;

        print_install_result(&result);
        Ok(())
    }
}
