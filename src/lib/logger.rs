use fern::colors::{Color, ColoredLevelConfig};
use log::Level;

use std::env;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .warn(Color::Yellow)
        .error(Color::BrightRed)
        .debug(Color::BrightCyan)
        .trace(Color::White)
        .info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "\x1B[{}m==> {}\x1B[0m{}",
                colors.get_color(&record.level()).to_fg_str(),
                match record.level() {
                    Level::Warn => "WARNING: ",
                    Level::Error => "ERROR: ",
                    Level::Debug => "DEBUG: ",
                    Level::Trace => "TRACE: ",
                    Level::Info => "",
                },
                msg
            ))
        })
        .level(match &*env::var("DEBUG").unwrap_or("0".to_string()) {
            "0" => log::LevelFilter::Info,
            _ => log::LevelFilter::Debug,
        })
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
