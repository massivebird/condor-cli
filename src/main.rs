use rand::Rng;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use time::UtcOffset;

fn main() {
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        simplelog::ConfigBuilder::new()
            .set_time_offset(UtcOffset::current_local_offset().unwrap())
            .build(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Always,
    )
    .unwrap();

    log::info!("Make sure to set your volume unmuted and low!");
    log::info!("Let me do my thing");

    let song_path = "/home/penguino/Music/Machine Girl/Wlfgrl/03 - Machine Girl - Krystle - URL Cyber Palace Mix.mp3";

    assert!(
        std::path::Path::new(song_path).exists(),
        "Song file {song_path} does not exist."
    );

    let crns: Vec<u32> = vec![
        11264, // STAT 360
        11265, // STAT 360
    ];

    let mut crn_iter = crns.iter().cycle();

    loop {
        let crn = crn_iter.next().unwrap();

        check_course(crn, &song_path);

        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_secs(rng.gen_range(30..=72)));
    }
}

fn check_course(crn: &u32, song_path: &str) {
    let alarm_on_loop = || loop {
        Command::new("mpv")
            .arg(song_path)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(240));
    };

    let course_catalog_url = format!("https://bannerweb.oci.emich.edu/pls/banner/bwckschd.p_disp_detail_sched?term_in=202510&crn_in={crn}");

    let html = String::from_utf8(
        Command::new("curl")
            .arg(course_catalog_url)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let html_re = regex::Regex::new(r"dddefault").unwrap();

    if !html_re.is_match(&html) {
        log::error!("Uh oh. We're not getting HTML anymore.");
        dbg!(html);
        return;
    }

    let availability_re = regex::Regex::new(r"dddefault.>[^30]{1,2}[0]{0,1}</td>").unwrap();

    if availability_re.is_match(&html) {
        log::warn!("NONTHIRTY, NONZERO VALUE DETECTED FOR COURSE {crn}!!!!");
        alarm_on_loop();
    }

    log::info!("CRN {crn} analysis clean.");
}
