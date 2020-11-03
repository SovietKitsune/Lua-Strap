extern crate git2;
extern crate hlua;
extern crate is_executable;
extern crate run_script;
extern crate serde;
extern crate toml;

pub mod dependencies; // Dependency installing
pub mod logger; // Logging with fern
pub mod task_runner; // Types like config
pub mod types; // To run tasks

mod lua;
