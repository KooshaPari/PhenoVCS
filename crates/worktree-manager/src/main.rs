//! worktree-manager CLI entry point

use clap::Parser;
use tracing::{error, info, warn};

use worktree_manager::{
    application::WorktreeService,
    cli::{Cli, Commands},
    domain::BranchName,
    infrastructure::GitWorktreeAdapter,
};

fn main() -> anyhow::Result<()> {
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
            info!(count = listing.worktrees.len(), "listing worktrees");

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
            info!(branch = %branch_name.as_str(), path = %path.display(), "creating worktree");
            let result =
                service.create_worktree(&repo_path, branch_name, &path, start_point.as_deref());

            if result.success {
                let wt_path = result
                    .worktree
                    .as_ref()
                    .map(|w| w.path.display().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                info!(path = %wt_path, "worktree created successfully");
                println!("Created worktree: {}", wt_path);
                for warning in result.warnings {
                    warn!(warning = %warning, "worktree creation warning");
                    eprintln!("Warning: {}", warning);
                }
            } else {
                let err_msg = result.error.as_deref().unwrap_or("unknown error");
                error!(error = %err_msg, "worktree creation failed");
                eprintln!("Failed: {}", err_msg);
                std::process::exit(1);
            }
        }

        Commands::Remove { path, force } => {
            info!(path = %path.display(), force, "removing worktree");
            service.remove_worktree(&path, force)?;
            println!("Removed worktree: {}", path.display());
        }

        Commands::Lock { path, reason } => {
            info!(path = %path.display(), reason = %reason, "locking worktree");
            service.lock_worktree(&path, &reason)?;
            println!("Locked: {}", path.display());
        }

        Commands::Unlock { path } => {
            info!(path = %path.display(), "unlocking worktree");
            service.unlock_worktree(&path)?;
            println!("Unlocked: {}", path.display());
        }

        Commands::Prune => {
            info!("pruning stale worktree references");
            service.prune(&repo_path)?;
            println!("Pruned stale worktree references");
        }

        Commands::Branch => {
            let branch = service.current_branch(&repo_path)?;
            info!(branch = %branch.as_str(), "current branch");
            println!("{}", branch.as_str());
        }

        // Cleanup commands - simplified, just prune for now
        Commands::Cleanup { dry_run, .. } => {
            if dry_run {
                info!("cleanup dry run - no changes made");
                println!("Dry run - no changes made");
            } else {
                info!("running cleanup (prune)");
                service.prune(&repo_path)?;
                println!("Pruned stale worktree references");
            }
        }
    }

    Ok(())
}
