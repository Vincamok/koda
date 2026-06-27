use std::path::Path;

use anyhow::Context;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use uuid::Uuid;

pub struct CloneResult {
    pub commit_sha: String,
    pub commit_message: String,
}

/// Clone or update a repository into the workspace volume path.
pub fn clone_repo(
    repo_url: &str,
    branch: &str,
    target_dir: &Path,
    ssh_key_pem: Option<&str>,
) -> anyhow::Result<CloneResult> {
    if target_dir.exists() && target_dir.join(".git").exists() {
        // Already cloned — pull latest
        return pull_repo(target_dir, branch, ssh_key_pem);
    }

    let mut callbacks = RemoteCallbacks::new();
    setup_credentials(&mut callbacks, ssh_key_pem);

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    fetch_opts.depth(50);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);
    builder.branch(branch);

    let repo = builder
        .clone(repo_url, target_dir)
        .with_context(|| format!("clone {repo_url} branch={branch}"))?;

    head_info(&repo)
}

fn pull_repo(dir: &Path, branch: &str, ssh_key_pem: Option<&str>) -> anyhow::Result<CloneResult> {
    let repo = Repository::open(dir).context("open existing repo")?;

    let mut remote = repo.find_remote("origin").context("find remote origin")?;

    let mut callbacks = RemoteCallbacks::new();
    setup_credentials(&mut callbacks, ssh_key_pem);
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote
        .fetch(&[branch], Some(&mut fetch_opts), None)
        .context("fetch origin")?;

    let fetch_head = repo.find_reference("FETCH_HEAD").context("FETCH_HEAD")?;
    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .context("annotated commit")?;

    let (analysis, _) = repo.merge_analysis(&[&fetch_commit]).context("merge analysis")?;

    if analysis.is_fast_forward() {
        let refname = format!("refs/heads/{branch}");
        let mut reference = repo.find_reference(&refname).context("branch ref")?;
        reference
            .set_target(fetch_commit.id(), "Fast-forward")
            .context("fast-forward")?;
        repo.set_head(&refname).context("set head")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .context("checkout head")?;
    } else if analysis.is_up_to_date() {
        tracing::debug!("repo is up to date");
    }

    head_info(&repo)
}

fn setup_credentials(callbacks: &mut RemoteCallbacks<'_>, ssh_key_pem: Option<&str>) {
    if let Some(pem) = ssh_key_pem {
        let pem_owned = pem.to_owned();
        callbacks.credentials(move |_url, username, _allowed| {
            Cred::ssh_key_from_memory(
                username.unwrap_or("git"),
                None,
                &pem_owned,
                None,
            )
        });
    } else {
        callbacks.credentials(|_url, username, _allowed| {
            Cred::ssh_key_from_agent(username.unwrap_or("git"))
        });
    }
}

fn head_info(repo: &Repository) -> anyhow::Result<CloneResult> {
    let head = repo.head().context("repo head")?;
    let commit = head.peel_to_commit().context("peel to commit")?;
    Ok(CloneResult {
        commit_sha: commit.id().to_string(),
        commit_message: commit.summary().unwrap_or("").to_owned(),
    })
}
