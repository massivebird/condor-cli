use std::path::Path;

use clap::{command, Arg, ValueHint};
use playback_rs::Song;
use time::UtcOffset;

#[derive(Clone)]
pub struct Config {
    pub verbose: bool,
    pub alarm: Song,
}

pub fn generate_config() -> Config {
    let matches = command!()
        .arg(
            Arg::new("alert")
                .short('r')
                .long("alert")
                .alias("a")
                .required(true)
                .help("Path to your alert sound")
                .value_name("PATH")
                .value_hint(ValueHint::DirPath),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Display additional output messages"),
        )
        .get_matches();

    let alert_path = matches.get_one::<String>("alert").unwrap();

    let Ok(alert) = playback_rs::Song::from_file(alert_path, None) else {
        panic!("Failed to parse alert file for some reason. Oops! >v<");
    };

    assert!(
        Path::new(alert_path).exists(),
        "Alert file {alert_path} does not exist."
    );

    // Oh yeah, we can configure our logger here, too.
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        simplelog::ConfigBuilder::new()
            .set_time_offset(UtcOffset::current_local_offset().unwrap())
            .build(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Always,
    )
    .unwrap();

    Config {
        verbose: matches.get_flag("verbose"),
        alarm: alert,
    }
}
