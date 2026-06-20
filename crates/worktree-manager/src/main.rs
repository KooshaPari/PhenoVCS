//! worktree-manager CLI entry point

use clap::Parser;
use phenovcs_observability::{self, SERVICE_NAME};

use worktree_manager::{
    application::WorktreeService,
    cli::{Cli, Commands},
    domain::BranchName,
    infrastructure::GitWorktreeAdapter,
};

fn main() -> anyhow::Result<()> {
    let otlp_endpoint = phenovcs_observability::otlp_endpoint();
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("service".to_string(), SERVICE_NAME.to_string());
    attrs.insert("otlp_endpoint".to_string(), otlp_endpoint.clone());
    // Emit the OTLP span synchronously via a one-shot tokio runtime.
    // This avoids blocking startup while still producing a span before any
    // user-visible work happens.
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(phenovcs_observability::emit_span("phenovcs.start", attrs));
    tracing::info!(service = SERVICE_NAME, otlp_endpoint = %otlp_endpoint, "phenovcs starting");

    let cli = Cli::parse();
    // Use clap-ext's tracing setup. Returns anyhow::Result because the
    // worktree-manager domain errors (WorktreeError) are not part of
    // clap_ext's CliError enum and forcing a hand-rolled From impl would
    // leak domain types into the shared library.
    clap_ext::prelude::setup_tracing(cli.verbosity.to_filter());

    // Determine repo path
    let repo_path = cli.repo.map(Ok).unwrap_or_else(std::env::current_dir)?;

    // Create adapter and service
    let adapter = GitWorktreeAdapter::new();
    let service = WorktreeService::new(adapter.clone(), adapter);

    match cli.command {
        Commands::List { json } => {
            let listing = service.list_worktrees(&repo_path)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&listing)?);
            } else {
                println!("Main: {}", listing.main.path.display());
                println!("Worktrees ({}):", listing.worktrees.len());
                for wt in &listing.worktrees {
                    let status = if wt.locked { "🔒" } else { "   " };
                    println!(
                        "  {} {} -> {}",
                        status,
                        wt.branch.as_str(),
                        wt.path.display()
                    );
                }
            }
        }

        Commands::Create {
            branch,
            path,
            start_point,
        } => {
            let branch_name = BranchName::new(branch);
            let result =
                service.create_worktree(&repo_path, branch_name, &path, start_point.as_deref());

            if result.success {
                println!(
                    "Created worktree: {}",
                    result.worktree.unwrap().path.display()
                );
                for warning in result.warnings {
                    eprintln!("Warning: {}", warning);
                }
            } else {
                eprintln!("Failed: {}", result.error.unwrap());
                std::process::exit(1);
            }
        }

        Commands::Remove { path, force } => {
            service.remove_worktree(&path, force)?;
            println!("Removed worktree: {}", path.display());
        }

        Commands::Lock { path, reason } => {
            service.lock_worktree(&path, &reason)?;
            println!("Locked: {}", path.display());
        }

        Commands::Unlock { path } => {
            service.unlock_worktree(&path)?;
            println!("Unlocked: {}", path.display());
        }

        Commands::Prune => {
            service.prune(&repo_path)?;
            println!("Pruned stale worktree references");
        }

        Commands::Branch => {
            let branch = service.current_branch(&repo_path)?;
            println!("{}", branch.as_str());
        }

        // Cleanup commands - simplified, just prune for now
        Commands::Cleanup { dry_run, .. } => {
            if dry_run {
                println!("Dry run - no changes made");
            } else {
                service.prune(&repo_path)?;
                println!("Pruned stale worktree references");
            }
        }
    }

    Ok(())
}
