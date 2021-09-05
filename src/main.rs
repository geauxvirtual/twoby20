// 2by20 is an indoor cycling training application with a simple of goal
// recoding an indoor training ride reading data from devices such as power
// meters, heart rate monitors, cadence sensors, etc.  and displaying this data
// on screen based on a predescribed workout. The workout will be recorded and
// be able to be exported to a .fit file for uploading to external applications
// or sites.
use std::str::FromStr;

use clap::{App, Arg};

// Configure command line options for the application.
fn app() -> App<'static, 'static> {
    App::new("2by20").arg(
        Arg::with_name("log-level")
            .short("l")
            .long("log-level")
            .value_name("LOG_LEVEL")
            .takes_value(true),
    )
}

fn main() {
    // Read command line options
    let matches = app().get_matches();
    // Default application logging to info
    let log_level = matches.value_of("log-level").unwrap_or("info");
    let log_level_filter = log::LevelFilter::from_str(log_level).unwrap();
    // Configure application level logging.
    // TODO: Switch from stdout logging to logging to file once the
    // application is stable enough.
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level_filter)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
