//! # Dependencies
//!
//! Dependencies manager to make sure dependencies are installed.

use std::{collections::HashMap, env, error, fmt, fs, io, path};

use super::types;

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, Progress, RemoteCallbacks};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use is_executable::IsExecutable;

#[derive(Debug)]
pub enum Error {
    GitError(git2::Error),
    IoError(io::Error),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GitError(e) => write!(f, "{}", e),
            Error::IoError(e) => write!(f, "{}", e),
        }
    }
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::GitError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

pub struct State {
    pub progress: Option<Progress<'static>>,
    pub total: usize,
    pub current: usize,
    pub path: Option<PathBuf>,
    pub newline: bool,
}

pub fn install_git_dep<F: Fn(&mut State) -> ()>(
    (location, details): (&String, &types::GitDependency),
    print: F,
) -> Result<(), Error> {
    // TODO; store in a cache located somewhere like ~/.lua-strap

    let to_save = details.path.as_ref().unwrap_or(location);

    let state = RefCell::new(State {
        progress: None,
        total: 0,
        current: 0,
        path: None,
        newline: false,
    });

    let mut cb = RemoteCallbacks::new();

    cb.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        state.progress = Some(stats.to_owned());
        print(&mut *state);
        true
    });

    let mut co = CheckoutBuilder::new();

    co.progress(|path, cur, total| {
        let mut state = state.borrow_mut();

        state.path = path.map(|p| p.to_path_buf());
        state.current = cur;
        state.total = total;

        print(&mut *state);
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    let repo = RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(&*details.source, Path::new(to_save))?;

    fs::remove_dir_all(repo.path())?;

    Ok(())
}

pub fn get_lit_install(deps: HashMap<String, types::LitDependency>) -> Vec<String> {
    let mut strs: Vec<String> = Vec::new(); // TODO; string concatenation?

    for (name, details) in deps.iter() {
        if details.version.as_ref().unwrap_or(&"latest".to_string()) == "latest" {
            strs.push(format!("{}/{} ", details.author, name));
        } else {
            strs.push(format!(
                "{}/{}@{} ",
                details.author,
                name,
                details.version.as_ref().unwrap()
            ));
        }
    }

    strs
}

pub fn is_in_path(file: &str) -> bool {
    if let Ok(path_env) = env::var("PATH") {
        let split = match cfg!(unix) {
            true => path_env.split(":"),
            false => path_env.split(";"),
        };

        for p in split {
            let p_str = &format!("{}/{}", p, file);
            let location = path::Path::new(p_str);

            if location.exists() && location.is_executable() {
                return true;
            }
        }
    }

    false
}
