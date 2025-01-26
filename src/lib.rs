#![allow(dead_code)]

use core::fmt;
use regex::Regex;
use std::error::Error;

#[derive(Debug)]
pub struct CourseStatus {
    pub title: String,
    pub code: String,
    pub actual_capacity: u32,
    pub actual_students: u32,
    pub actual_remaining: u32,
    pub waitlist_capacity: u32,
    pub waitlist_students: u32,
    pub waitlist_remaining: u32,
    pub cross_list_capacity: u32,
    pub cross_list_students: u32,
    pub cross_list_remaining: u32,
}

impl CourseStatus {
    /// Returns `true` if there are actual seats remaining. Returns `false` otherwise.
    #[must_use]
    pub const fn has_open_seats(&self) -> bool {
        self.actual_remaining > 0
    }

    /// Returns `true` if there are waitlist seats remaining. Returns `false` otherwise.
    #[must_use]
    pub const fn has_open_waitlist(&self) -> bool {
        self.waitlist_remaining > 0
    }

    /// Returns `true` if there are actual seats or waitlist seats remaining. Returns `false` otherwise.
    #[must_use]
    pub const fn has_open_anything(&self) -> bool {
        self.has_open_seats() || self.has_open_waitlist()
    }
}

#[derive(Debug)]
pub struct CaptureGenError {
    pub crn: String,
}

impl CaptureGenError {
    fn new(crn: &str) -> Self {
        Self {
            crn: crn.to_string(),
        }
    }
}

impl fmt::Display for CaptureGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to generate regex captures for CRN {}", self.crn)
    }
}

impl Error for CaptureGenError {}

pub fn get_course_status(crn: &str, semester_code: &str) -> Result<CourseStatus, Box<dyn Error>> {
    let catalog_url = format!("https://bannerweb.oci.emich.edu/pls/banner/bwckschd.p_disp_detail_sched?term_in={semester_code}&crn_in={crn}");

    let html = reqwest::blocking::get(catalog_url)?.text()?;

    let regex = Regex::new(
        r#"<th CLASS=\"ddlabel\" scope=\"row\" >(?<title>.*) - \d{5} - (?<code>[\w\s\d]+) - \d+<br /><br /></th>"#,
    )?;

    let Some(captures) = regex.captures(&html) else {
        return Err(CaptureGenError::new(crn).into());
    };

    let title = captures.name("title").unwrap().as_str();
    let code = captures.name("code").unwrap().as_str();

    let regex = Regex::new(
        r#"Seats</SPAN></th>\n<td CLASS=\"dddefault\">(?<actual_capacity>\d+)</td>\n<td CLASS=\"dddefault\">(?<actual_students>\d+)</td>\n<td CLASS=\"dddefault\">(?<actual_remaining>-?\d+)</td>\n</tr>\n<tr>\n<th CLASS=\"ddlabel\" scope=\"row\" ><SPAN class=\"fieldlabeltext\">Waitlist Seats</SPAN></th>\n<td CLASS=\"dddefault\">(?<waitlist_capacity>\d+)</td>\n<td CLASS=\"dddefault\">(?<waitlist_students>\d+)</td>\n<td CLASS=\"dddefault\">(?<waitlist_remaining>-?\d+)</td>\n</tr>\n<tr>\n<th CLASS=\"ddlabel\" scope=\"row\" ><SPAN class=\"fieldlabeltext\">Cross List Seats</SPAN></th>\n<td CLASS=\"dddefault\">(?<cross_list_capacity>-?\d+)</td>\n<td CLASS=\"dddefault\">(?<cross_list_students>-?\d+)</td>\n<td CLASS=\"dddefault\">(?<cross_list_remaining>-?\d+)</td>"#,
    )?;

    let Some(captures) = regex.captures(&html) else {
        dbg!(&html);
        return Err(CaptureGenError::new(crn).into());
    };

    // Attempts to retrieve a named capture. Returns from the function on failure.
    macro_rules! try_get_capture {
        ( $x: expr ) => {{
            captures.name($x).unwrap().as_str().parse()?
        }};
    }

    let actual_capacity: u32 = captures.name("actual_capacity").unwrap().as_str().parse()?;
    let actual_students: u32 = try_get_capture!("actual_students");
    let actual_remaining: u32 = actual_capacity.saturating_sub(actual_students);

    let waitlist_capacity: u32 = try_get_capture!("waitlist_capacity");
    let waitlist_students: u32 = try_get_capture!("waitlist_students");
    let waitlist_remaining: u32 = waitlist_capacity.saturating_sub(waitlist_students);

    let cross_list_capacity: u32 = try_get_capture!("cross_list_capacity");
    let cross_list_students: u32 = try_get_capture!("cross_list_students");
    let cross_list_remaining: u32 = cross_list_capacity.saturating_sub(cross_list_students);

    Ok(CourseStatus {
        title: title.to_string(),
        code: code.to_string(),
        actual_capacity,
        actual_students,
        actual_remaining,
        waitlist_capacity,
        waitlist_students,
        waitlist_remaining,
        cross_list_capacity,
        cross_list_students,
        cross_list_remaining,
    })
}
