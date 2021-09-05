// 2by20 is an indoor cycling training application with a simple of goal
// recoding an indoor training ride reading data from devices such as power
// meters, heart rate monitors, cadence sensors, etc.  and displaying this data
// on screen based on a predescribed workout. The workout will be recorded and
// be able to be exported to a .fit file for uploading to external applications
// or sites.

fn main() {
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
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
