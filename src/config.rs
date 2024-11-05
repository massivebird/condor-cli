use std::path::Path;

use clap::{command, Arg, ValueHint};
use playback_rs::Song;
use time::UtcOffset;

#[derive(Clone)]
pub struct Config {
    pub verbose: bool,
    pub alarm: Song,
    pub crns: Vec<u32>,
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
            Arg::new("crns")
                .short('c')
                .long("crns")
                .required(true)
                .help("Comma-separated CRNs to monitor.")
                .value_name("CRNs"),
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

    log::info!("Generating configuration. This might take a while.");
    log::info!("(Make sure your volume is comfortable!)");

    let crns: Vec<u32> = matches
        .get_one::<String>("crns")
        .map(|labels| {
            labels
                .split(',')
                .map(|crn| {
                    crn.parse::<u32>().unwrap_or_else(|_| {
                        log::error!("ERROR: could not parse CRN `{crn}` into an integer.");
                        std::process::exit(1);
                    })
                })
                .collect()
        })
        .unwrap();

    let alert_path = matches.get_one::<String>("alert").unwrap();

    let Ok(alert) = playback_rs::Song::from_file(alert_path, None) else {
        panic!("Failed to parse alert file for some reason. Oops! >v<");
    };

    assert!(
        Path::new(alert_path).exists(),
        "Alert file {alert_path} does not exist."
    );

    Config {
        verbose: matches.get_flag("verbose"),
        alarm: alert,
        crns,
    }
}
