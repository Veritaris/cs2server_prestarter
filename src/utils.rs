pub fn seconds_to_minutes_str(seconds: u32) -> String {
    let minutes: f64 = seconds as f64 / 60.0;
    let seconds_partial: f64 = ((seconds as f64 - minutes * 60.0)) / 60.0;
    return format!("{:.2}", minutes + seconds_partial);
}

pub fn bool_to_str(val: bool) -> String {
    return String::from(if val {
        "1"
    } else {
        "0"
    });
}