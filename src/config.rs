use clap::{command, Arg, ValueHint};
use playback_rs::Song;
use regex::Regex;
use std::fs;
use std::path::Path;
use time::UtcOffset;

#[derive(Clone)]
pub struct Config {
    pub verbose: bool,
    pub alarm: Song,
    pub crns: Vec<u32>,
    pub semester_code: String,
}

pub fn generate_config() -> Config {
    let matches = command!()
        .arg(
            Arg::new("alert")
                .short('a')
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
            Arg::new("semester")
                .short('s')
                .long("semester")
                .required(true)
                .help("Semester name: formatted \"fall2024\" or \"winter2025\"")
                .value_name("STRING"),
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

    // Logger configuration. Basically boilerplate
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

    let semester_code = {
        let input = matches.get_one::<String>("semester").unwrap();
        let re = Regex::new(r#"(?<season>fall|winter)(?<year>20\d{2})"#).unwrap();

        let error_msg =
            "Failed to parse semester: `<fall|winter><year>` expected, such as \"fall2024\".";

        let Some(captures) = re.captures(input) else {
            log::error!("{}", error_msg);
            std::process::exit(1);
        };

        // TODO: a similar macro appears in other files. Dissolve them!
        macro_rules! try_get_capture {
            ( $x: expr, $error_msg: expr ) => {{
                captures.name($x).unwrap().as_str()
            }};
        }

        let season = try_get_capture!("season", error_msg);
        let year = try_get_capture!("year", error_msg).parse::<u32>().unwrap();

        match season {
            "fall" => format!("{}10", year + 1),
            "winter" => format!("{year}20"),
            _ => unreachable!(),
        }
    };

    let alert_path = matches.get_one::<String>("alert").unwrap();

    match fs::exists(alert_path) {
        Ok(true) => (), // File exists
        Ok(false) => {
            log::error!("Alert file does not exist.");
            std::process::exit(1);
        }
        Err(_) => {
            log::error!("Failed to access alert file. Maybe a permissions issue?");
            std::process::exit(1);
        }
    }

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
        semester_code,
    }
}
