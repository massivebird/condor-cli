use clap::{command, Arg, ArgMatches, ValueHint};

pub fn generate_matches() -> ArgMatches {
    command!()
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
        .get_matches()
}
