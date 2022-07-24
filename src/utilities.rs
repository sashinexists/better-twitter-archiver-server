use chrono::{DateTime, FixedOffset};
use rocket::{
    serde::Serialize,
    time::{format_description, OffsetDateTime},
};

pub fn convert_date_to_chrono(date: Option<OffsetDateTime>) -> DateTime<FixedOffset> {
    let format = format_description::parse(
        "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour \
             sign:mandatory]:[offset_minute]",
    )
    .expect("Bad formatter");

    let date_string = date
        .expect("Couldn't get the tweets date")
        .format(&format)
        .expect("Couldn't parse with thes formatter");

    chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&date_string)
        .expect("failed to parse date from string")
}

pub fn to_ron<T: ?Sized + Serialize>(item: &T) -> String {
    ron::ser::to_string_pretty(item, ron::ser::PrettyConfig::new())
        .expect("Failed to parse tweet into string")
}
