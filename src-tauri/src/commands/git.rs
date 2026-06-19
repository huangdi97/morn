use serde::Serialize;
use std::process::Command;
use tauri::State;

use crate::AppState;

#[derive(Serialize, Clone)]
pub(crate) struct GitCommit {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub time: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct GitInfo {
    pub repo_path: String,
    pub branch: String,
    pub uncommitted_changes: usize,
    pub recent_commits: Vec<GitCommit>,
    pub is_git_repo: bool,
    pub error: Option<String>,
}

fn run_git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
}

#[tauri::command]
pub(crate) fn git_info(_state: State<AppState>) -> GitInfo {
    let cwd = std::env::current_dir().unwrap_or_default();

    let is_git = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_git {
        return GitInfo {
            repo_path: cwd.to_string_lossy().to_string(),
            branch: String::new(),
            uncommitted_changes: 0,
            recent_commits: vec![],
            is_git_repo: false,
            error: None,
        };
    }

    let repo_path = run_git(&["rev-parse", "--show-toplevel"]).unwrap_or_default();
    let branch = run_git(&["branch", "--show-current"]).unwrap_or_default();
    let uncommitted_changes = run_git(&["status", "--porcelain"])
        .map(|s| s.lines().count())
        .unwrap_or(0);

    let log_output =
        run_git(&["log", "--oneline", "-5", "--format=%H|%an|%s|%ar"]).unwrap_or_default();

    let recent_commits: Vec<GitCommit> = log_output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                Some(GitCommit {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    message: parts[2].to_string(),
                    time: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    GitInfo {
        repo_path,
        branch,
        uncommitted_changes,
        recent_commits,
        is_git_repo: true,
        error: None,
    }
}
