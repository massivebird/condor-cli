use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    let my_print = |msg: &str| {
        let timestamp = chrono::Local::now();
        println!("[{timestamp:?}] {msg}");
    };
    my_print("Make sure to set your volume unmuted and low!");
    my_print("Let me do my thing");

    let alarm_on_loop = || loop {
        Command::new("mpv")
            .arg("/home/penguino/Music/Krystle (URL Cyber Palace Mix) [dVsdh98eapI].m4a")
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(240));
    };

    loop {
        let html = String::from_utf8(Command::new("curl").arg("https://bannerweb.oci.emich.edu/pls/banner/bwckschd.p_disp_detail_sched?term_in=202510&crn_in=13153").output().unwrap().stdout).unwrap();

        let re = regex::Regex::new(r"dddefault").unwrap();

        if !re.is_match(&html) {
            my_print("Uh oh. We're not getting HTML anymore.");
            continue;
        }

        let re = regex::Regex::new(r"dddefault.>[^30]{1,2}[0]{0,1}</td>").unwrap();

        if re.is_match(&html) {
            my_print("NONTHIRTY, NONZERO VALUE DETECTED!!!!");
            alarm_on_loop();
        }

        thread::sleep(Duration::from_secs(60));
    }
}
