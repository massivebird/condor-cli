use playback_rs::Song;
use regex::Regex;
use std::{fs, process};
use std::path::Path;
use time::UtcOffset;

mod cli;

#[derive(Clone)]
pub struct Config {
    pub verbose: bool,
    pub alarm: Song,
    pub crns: Vec<u32>,
    pub semester_code: String,
}

pub fn generate_config() -> Config {
    let matches = cli::generate_matches();

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
                .split(&[',', ' '][..])
                .map(|crn| {
                    crn.parse::<u32>().unwrap_or_else(|_| {
                        log::error!("ERROR: could not parse CRN `{crn}` into an integer.");
                        process::exit(1);
                    })
                })
                .collect()
        })
        .unwrap();

    let semester_code = parse_semester(matches.get_one::<String>("semester").unwrap());

    let alert_path = matches.get_one::<String>("alert").unwrap();

    match fs::exists(alert_path) {
        Ok(true) => (), // File exists
        Ok(false) => {
            log::error!("Alert file does not exist.");
            process::exit(1);
        }
        Err(_) => {
            log::error!("Failed to access alert file. Maybe a permissions issue?");
            process::exit(1);
        }
    }

    let Ok(alert) = playback_rs::Song::from_file(alert_path, None) else {
        panic!("Failed to parse alert file for some reason. Oops! >v<");
    };

    Config {
        verbose: matches.get_flag("verbose"),
        alarm: alert,
        crns,
        semester_code,
    }
}

fn parse_semester(input: &str) -> String {
    let re = Regex::new(r"(?<season>fall|winter)(?<year>20\d{2})").unwrap();

    let Some(captures) = re.captures(input) else {
        log::error!("{}", "Failed to parse semester: `<fall|winter><year>` expected, such as \"fall2024\".");
        process::exit(1);
    };

    // TODO: a similar macro appears in other files. Dissolve them!
    macro_rules! try_get_capture {
        ( $x: expr, $error_msg: expr ) => {{
            captures.name($x).unwrap().as_str()
        }};
    }

    let season = try_get_capture!("season", error_msg);
    let year = try_get_capture!("year", error_msg).parse::<u32>().unwrap();

    // term_in values:
    // Winter 2025   202520
    // Fall 2024     202510
    // Winter 2024   202420
    // Fall 2024     202410
    match season {
        "fall" => format!("{}10", year + 1),
        "winter" => format!("{year}20"),
        _ => unreachable!(),
    }
}
