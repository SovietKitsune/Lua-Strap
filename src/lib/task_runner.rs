use super::types::{Runner, Runtime, Task};

use hlua::Lua;

use run_script::{run, types::ScriptError, types::ScriptOptions};

use std::{env::consts::OS, error, fmt, fs, io};

#[derive(Debug)]
pub enum Error {
  ShellError(ScriptError),
  IoError(io::Error),
  RuntimeError,
  NoFileFound,
}

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::ShellError(e) => write!(f, "{}", e),
      Error::IoError(e) => write!(f, "{}", e),
      Error::RuntimeError => write!(f, "The task failed at runtime"),
      Error::NoFileFound => write!(f, "No file was found to run"),
    }
  }
}

impl From<ScriptError> for Error {
  fn from(e: ScriptError) -> Self {
    Error::ShellError(e)
  }
}

impl From<io::Error> for Error {
  fn from(e: io::Error) -> Self {
    Error::IoError(e)
  }
}

// Resolves files if the task uses them
fn resolve(file: Option<&String>) -> Result<String, Error> {
  if file.is_none() {
    return Err(Error::NoFileFound);
  }

  let name = file.unwrap();

  Ok(String::from_utf8_lossy(&fs::read(name)?).to_string())
}

pub fn run_task(mut runner: Runner, task: &Task) -> Result<(), Error> {
  if runner.lua.is_none() {
    // Construct a Lua runtime
    runner.lua = Some(Lua::new());
  }

  if runner.options.is_none() {
    runner.options = Some(ScriptOptions::new())
  }

  // Now we know that we have the runtime setup
  // Shell is simple enough to not need its own engine

  let rt = &task.runtime;

  let script = match OS {
    "windows" => {
      if task.script_windows.is_none() {
        task.script.as_ref()
      } else {
        task.script_windows.as_ref()
      }
    }
    _ => task.script.as_ref(),
  };

  let to_run = match script {
    None => resolve(task.file.as_ref())?,
    _ => script.unwrap().to_string(),
  };

  match rt {
    Runtime::Shell => {
      let (code, output, error) = run(&to_run, &vec![], &runner.options.unwrap())?;

      println!("{}", output);

      if code != 0 {
        error!("Failed to complete task\n{}", error);

        return Err(Error::RuntimeError);
      }
    }
    Runtime::Lua => error!("Lua is not implemented yet!"),
  };

  /*
  let (code, output, error) = run(resolve(task), vec![], &runner.options.unwrap())?;

  println!("{}", output);

  if code != 0 {
    error!("Failed to complete task\n{}", error);
  }
  */

  Ok(())
}
