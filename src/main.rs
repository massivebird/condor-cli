use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    let timestamp = chrono::Local::now();
    println!("[{timestamp:?}] Let me do my thing");

    loop {
        let html = String::from_utf8(Command::new("curl").arg("https://bannerweb.oci.emich.edu/pls/banner/bwckschd.p_disp_detail_sched?term_in=202510&crn_in=13153").output().unwrap().stdout).unwrap();

        let timestamp = chrono::Local::now();

        let re = regex::Regex::new(r"dddefault.>[^30]{1,2}[0]{0,1}</td>").unwrap();

        if re.is_match(&html) {
            println!("[{timestamp:?}] NONTHIRTY, NONZERO VALUE DETECTED!!!!");
        }

        thread::sleep(Duration::from_secs(60));
    }
}
