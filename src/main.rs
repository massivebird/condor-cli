use condor::{get_course_status, CourseStatus};
use config::Config;
use playback_rs::Player;
use rand::Rng;
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

    let course_status: CourseStatus =
        match get_course_status(&crn.to_string(), &config.semester_code) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{e}");
                return;
            }
        };

    if course_status.has_open_anything() {
        log::warn!(
            "Detected vacancy for {crn}! (Rem: {}, WLRem: {})",
            course_status.actual_remaining,
            course_status.waitlist_remaining
        );
        alarm_on_loop();
    }

    if config.verbose {
        log::info!("No vacancy detected for {crn}.");
    }
}
