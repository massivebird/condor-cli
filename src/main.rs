use config::Config;
use playback_rs::Player;
use rand::Rng;
use regex::Regex;
use std::thread;
use std::time::Duration;

mod config;

fn main() {
    let config = config::generate_config();

    let player = playback_rs::Player::new(None).unwrap();

    let mut crn_iter = config.crns.iter().cycle();

    loop {
        let crn = crn_iter.next().unwrap();

        check_course(&config, *crn, &player);

        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_secs(rng.gen_range(30..=72)));
    }
}

fn check_course(config: &Config, crn: u32, player: &Player) {
    let alarm_on_loop = || loop {
        player.play_song_now(&config.alarm, None).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(240));
    };

    let course_catalog_url = format!("https://bannerweb.oci.emich.edu/pls/banner/bwckschd.p_disp_detail_sched?term_in={}&crn_in={crn}", config.semester_code);

    let html = reqwest::blocking::get(course_catalog_url)
        .unwrap()
        .text()
        .unwrap();

    let regex = Regex::new(r#"Seats</SPAN></th>\n<td CLASS=\"dddefault\">\d{1,2}</td>\n<td CLASS=\"dddefault\">(?<actual>\d{1,2})</td>\n<td CLASS=\"dddefault\">(?<remaining>-?\d{1,2})</td>\n</tr>\n<tr>\n<th CLASS=\"ddlabel\" scope=\"row\" ><SPAN class=\"fieldlabeltext\">Waitlist Seats</SPAN></th>\n(<td CLASS=\"dddefault\">\d{1,2}</td>\n){2}<td CLASS=\"dddefault\">(?<waitlist_remaining>-?\d{1,2})</td>"#).unwrap();

    let Some(captures) = regex.captures(&html) else {
        log::error!("Unexpected HTML response for {crn}: failed to generate captures.");
        return;
    };

    // Attempts to retrieve a named capture. Returns from the function on failure.
    macro_rules! try_get_capture {
        ( $x: expr ) => {{
            let Ok(value) = captures.name($x).unwrap().as_str().parse() else {
                log::error!(
                    "Unexpected HTML response for {crn}: failed to parse capture to integer."
                );
                return;
            };

            value
        }};
    }

    let actual: i32 = try_get_capture!("actual");
    // Don't freak out if registration hasn't begun for this semester.
    if actual == 0 {
        log::warn!(
            "Detected zero \"actual\" seats for {crn}. Ignore if registration has not started."
        );
        return;
    }

    let remaining: i32 = try_get_capture!("remaining");
    let waitlist_remaining: i32 = try_get_capture!("waitlist_remaining");

    if remaining > 0 || waitlist_remaining > 0 {
        log::warn!("Detected vacancy for {crn}! (Rem: {remaining}, WLRem: {waitlist_remaining})");
        alarm_on_loop();
    }

    if config.verbose {
        log::info!(
            "No vacancy detected for {crn}. (Rem: {remaining}, WLRem: {waitlist_remaining})"
        );
    }
}
