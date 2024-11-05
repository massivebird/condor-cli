use clap::{command, Arg, ValueHint};
use rand::Rng;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use time::UtcOffset;

fn main() {
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

    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        simplelog::ConfigBuilder::new()
            .set_time_offset(UtcOffset::current_local_offset().unwrap())
            .build(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Always,
    )
    .unwrap();

    let get_arg = |arg_name: &str| -> Option<&String> { matches.get_one::<String>(arg_name) };

    let alert_path = get_arg("alert").unwrap();

    let Ok(alert) = playback_rs::Song::from_file(alert_path, None) else {
        panic!("Failed to detect song at {alert_path}");
    };

    assert!(
        Path::new(alert_path).exists(),
        "Alert file {alert_path} does not exist."
    );

    let crns: Vec<u32> = vec![
        11264, // STAT 360
        11265, // STAT 360
    ];

    log::info!("Make sure to set your volume unmuted and low!");
    log::info!("Let me do my thing");

    let mut crn_iter = crns.iter().cycle();

    loop {
        let crn = crn_iter.next().unwrap();

        check_course(*crn, alert_path);

        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_secs(rng.gen_range(30..=72)));
    }
}

fn check_course(crn: u32, song_path: &str) {
    let alarm_on_loop = || loop {
        Command::new("mpv")
            .arg(song_path)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(240));
    };

    let course_catalog_url = format!("https://bannerweb.oci.emich.edu/pls/banner/bwckschd.p_disp_detail_sched?term_in=202510&crn_in={crn}");

    let html = reqwest::blocking::get(course_catalog_url)
        .unwrap()
        .text()
        .unwrap();

    let regex = regex::Regex::new(r#"Seats</SPAN></th>\n(<td CLASS=\"dddefault\">(?<cap>\d{1,2})</td>\n){2}<td CLASS=\"dddefault\">(?<remaining>-?\d{1,2})</td>\n</tr>\n<tr>\n<th CLASS=\"ddlabel\" scope=\"row\" ><SPAN class=\"fieldlabeltext\">Waitlist Seats</SPAN></th>\n(<td CLASS=\"dddefault\">\d{1,2}</td>\n){2}<td CLASS=\"dddefault\">(?<waitlist_remaining>-?\d{1,2})</td>"#).unwrap();

    let Some(captures) = regex.captures(&html) else {
        log::error!("CRN {crn}: unexpected HTML response: failed to generate captures.");
        return;
    };

    // Attempts to retrieve a named capture. Returns from the function on failure.
    macro_rules! try_get_capture {
        ( $x: expr ) => {{
            let Ok(value) = captures.name($x).unwrap().as_str().parse() else {
                log::error!(
                    "CRN {crn}: unexpected HTML response: failed to parse capture to integer."
                );
                return;
            };

            value
        }};
    }

    let remaining: i32 = try_get_capture!("remaining");
    let waitlist_remaining: i32 = try_get_capture!("remaining");

    if remaining > 0 || waitlist_remaining > 0 {
        log::warn!("CRN {crn}: vacancy has been detected!");
        alarm_on_loop();
    }

    log::info!("CRN {crn}: no vacancy detected.");
}
