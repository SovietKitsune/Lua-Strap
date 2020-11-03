extern crate ctrlc;
extern crate git2;
extern crate is_executable;

mod lib;

use lib::{dependencies, logger, task_runner, types};

use std::{collections::HashMap, fs};

use indicatif::{ProgressBar, ProgressStyle};
use std::process::{exit, Command};

#[macro_use]
extern crate log;

fn main() {
    logger::setup_logger().unwrap();

    match ctrlc::set_handler(move || {
        error!("SIGINT received, exiting...");
        // TODO; run cleanup task if it exists
        exit(0);
    }) {
        Ok(_) => {}
        Err(_) => error!("Error setting Ctrl-C handler"),
    };

    // TODO; read from arguments
    let contents: String = String::from_utf8_lossy(&fs::read("test.toml").unwrap())
        .parse()
        .unwrap();

    let config =
        types::Config::new(&contents).expect("Invalid configuration, please check configuration!");

    info!(
        "Setting up {}@{}",
        config.package.name, config.package.version
    );

    let deps = config.dependencies.unwrap_or(types::Dependencies {
        bin: None,
        git: None,
        lit: None,
    });

    let bin_deps: Vec<String> = deps.bin.unwrap_or(vec![]);
    let git_deps: HashMap<String, types::GitDependency> = deps.git.unwrap_or(HashMap::new());
    let lit_deps: HashMap<String, types::LitDependency> = deps.lit.unwrap_or(HashMap::new());

    if bin_deps.len() > 0 {
        info!("Checking bin for dependencies");

        for dep in bin_deps.iter() {
            if !dependencies::is_in_path(&dep) {
                error!("Missing {}", dep);
                exit(127);
            } else {
                info!("{} located", dep);
            }
        }
    }

    if git_deps.len() > 0 {
        info!("Installing dependencies via git")
    }

    // TODO; multithread?
    for (location, details) in git_deps.iter() {
        let pb = ProgressBar::new(1);

        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.white} [{elapsed_precise}] {msg} [{wide_bar:.green}] {pos}/{len} ({eta})")
            .progress_chars("=> ")
        );

        match dependencies::install_git_dep((location, details), |state| {
            let stats = state.progress.as_ref().unwrap();

            if stats.received_objects() == stats.total_objects() {
                pb.set_message(&format!("Resolving deltas [{}]", location));

                pb.set_length(stats.total_deltas() as u64);
                pb.set_position(stats.indexed_deltas() as u64);
            } else {
                pb.set_message(&format!("Receiving objects [{}]", location));

                pb.set_length(stats.total_objects() as u64);
                pb.set_position(stats.received_objects() as u64);
            }
        }) {
            Ok(_) => pb.finish_with_message(&format!("Installed {}", location)),
            Err(e) => {
                // TODO; there must be a better way...
                match e {
                    dependencies::Error::GitError(err) => match err.code() {
                        git2::ErrorCode::Exists => {
                            pb.set_position(1);
                            pb.abandon_with_message(&format!("Already installed {}", location));
                        }
                        _ => {
                            pb.abandon_with_message("Failed to install dependency");
                            error!(
                                "An unexpected error happened while install dependencies. {}",
                                err
                            );
                            exit(1);
                        }
                    },
                    err => {
                        pb.abandon_with_message("Failed to install dependency");
                        error!(
                            "An unexpected error happened while install dependencies. {}",
                            err
                        );
                        exit(1);
                    }
                };
            }
        };
    }

    if lit_deps.len() > 0 {
        info!("Installing dependencies via lit");

        if !dependencies::is_in_path("lit") {
            error!("Lit is not installed and is required!");
            exit(127);
        }
    }

    // TODO; create mini-lit?
    match Command::new("lit")
        .arg("install")
        .args(dependencies::get_lit_install(lit_deps))
        .output()
    {
        Ok(out) => {
            println!(
                "\n{}",
                String::from_utf8(out.stdout).expect("Failed to read stdout")
            );

            if out.status.code().unwrap() != 0 {
                error!("Failed to install dependencies via lit, check logs above.");
            }
        }
        Err(e) => {
            error!("Failed to install dependencies via lit. {}", e);
            exit(1);
        }
    };

    info!("Running `main` task");

    let runner = types::Runner::new(&config.tasks);

    match task_runner::run_task(runner, config.tasks.get("main").unwrap()) {
        Err(e) => error!("Failed to run `main` task, {}", e),
        _ => info!("Finished `main` task"),
    };
}
