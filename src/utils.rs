use chrono::{DateTime, FixedOffset};
use rocket::{
    serde::Serialize,
    time::{format_description, OffsetDateTime},
    tokio::sync::watch::Ref,
};
use twitter_v2::data::ReferencedTweetKind;

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

pub struct TweetReferenceData {
    pub reference_type: ReferencedTweetKind,
    pub source_tweet_id: i64,
    pub reference_tweet_id: i64,
}

impl TweetReferenceData {
    pub fn type_to_string(&self) -> String {
        match self.reference_type {
            ReferencedTweetKind::RepliedTo => "replied_to",
            ReferencedTweetKind::Retweeted => "retweeted",
            ReferencedTweetKind::Quoted => "quoted",
        }
        .to_string()
    }

    pub fn kind_from_string(input: &str) -> Option<ReferencedTweetKind> {
        match input {
            "replied_to" => Some(ReferencedTweetKind::RepliedTo),
            "retweeted" => Some(ReferencedTweetKind::Retweeted),
            "quoted" => Some(ReferencedTweetKind::Quoted),
            _ => None,
        }
    }
}
