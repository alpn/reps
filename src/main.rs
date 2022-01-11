use git2;
use log::error;
use std::env;

#[derive(PartialEq, Copy, Clone)]
pub enum GitRepoStatus {
    NoRepo,
    GitClean,
    GitDirty,
}

#[derive(Clone)]
pub struct GitRepo {
    pub status: GitRepoStatus,
    pub current_branch: Option<String>,
}

impl Default for GitRepo {
    fn default() -> Self {
        Self {
            status: GitRepoStatus::NoRepo,
            current_branch: None,
        }
    }
}

fn current_branch(repo: &git2::Repository) -> Option<String> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e)
            if e.code() == git2::ErrorCode::UnbornBranch
                || e.code() == git2::ErrorCode::NotFound =>
        {
            return None
        }
        Err(e) => {
            panic!("Error looking up Git branch: {:?}", e);
        }
    };

    if let Some(h) = head {
        if let Some(s) = h.shorthand() {
            let branch_name = s.to_owned();
            if branch_name.len() > 10 {
                return Some(branch_name[..8].to_string() + "..");
            }
            return Some(branch_name);
        }
    }
    None
}

impl GitRepo {
    pub fn get_status(path: &String) -> Self {
        let g = git2::Repository::open(path);
        if let Ok(repo) = g {
            let branch = current_branch(&repo);
            match repo.statuses(None) {
                Ok(es) => {
                    if es
                        .iter()
                        .filter(|s| s.status() != git2::Status::IGNORED)
                        .any(|_| true)
                    {
                        return Self {
                            status: GitRepoStatus::GitDirty,
                            current_branch: branch,
                        };
                    }
                    return Self {
                        status: GitRepoStatus::GitClean,
                        current_branch: branch,
                    };
                }
                Err(e) => {
                    error!("Error looking up Git statuses: {:?}", e);
                }
            }
        }
        Self::default()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    println!("Getting info for {:?}", path);

    let repo_status = GitRepo::get_status(path);
    match repo_status.status {
        GitRepoStatus::NoRepo => println!("not a repo"),
        GitRepoStatus::GitClean => {
            println!("clean");
            println!("branch: {}", repo_status.current_branch.unwrap());
        }
        GitRepoStatus::GitDirty => {
            println!("dirty");
            println!("branch: {}", repo_status.current_branch.unwrap());
        }
    }
}
