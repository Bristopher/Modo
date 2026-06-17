// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    SKIP_TS_PREPARE_ENV, TS_REFERENCE_COMMIT, TS_WORKTREE_ENV, TS_WORKTREE_ROOT_ENV, env_flag,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn ensure_reference_worktree(repo_root: &Path) -> Result<PathBuf, String> {
    let target_commit = git_stdout(repo_root, &["rev-parse", TS_REFERENCE_COMMIT])?;
    if let Ok(path) = env::var(TS_WORKTREE_ENV) {
        let path = PathBuf::from(path);
        validate_worktree(&path, &target_commit)?;
        return Ok(path);
    }
    let root = env::var(TS_WORKTREE_ROOT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root.join("target/parity"));
    fs::create_dir_all(&root)
        .map_err(|error| format!("failed to create {}: {error}", root.display()))?;
    let worktree = root.join("fluxer-admin-ts-ref-4748f2f1-parent");
    if !worktree.exists() {
        run_git(
            repo_root,
            &[
                "worktree",
                "add",
                "--detach",
                path_arg(&worktree).as_str(),
                TS_REFERENCE_COMMIT,
            ],
        )?;
    }
    validate_worktree(&worktree, &target_commit)?;
    Ok(worktree)
}

pub fn prepare_reference_package(worktree: &Path) -> Result<(), String> {
    if env_flag(SKIP_TS_PREPARE_ENV) {
        return Ok(());
    }
    run_command(
        Command::new("pnpm")
            .current_dir(worktree)
            .args(["install", "--frozen-lockfile"]),
        "pnpm install --frozen-lockfile",
    )?;
    run_command(
        Command::new("pnpm")
            .current_dir(worktree)
            .args(["--filter", "@fluxer/config", "generate"]),
        "pnpm --filter @fluxer/config generate",
    )?;
    run_command(
        Command::new("pnpm")
            .current_dir(worktree)
            .args(["--filter", "fluxer_admin", "build:css"]),
        "pnpm --filter fluxer_admin build:css",
    )
}

fn validate_worktree(worktree: &Path, target_commit: &str) -> Result<(), String> {
    if !worktree.join("fluxer_admin/package.json").exists() {
        return Err(format!(
            "{} does not look like the TS reference worktree",
            worktree.display()
        ));
    }
    let head = git_stdout(worktree, &["rev-parse", "HEAD"])?;
    if head != target_commit {
        return Err(format!(
            "{} is at {head}, expected {target_commit}",
            worktree.display()
        ));
    }
    Ok(())
}

fn run_git(repo: &Path, args: &[&str]) -> Result<(), String> {
    run_command(Command::new("git").current_dir(repo).args(args), "git")
}

fn git_stdout(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn run_command(command: &mut Command, label: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|error| format!("failed to run {label}: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "{label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
