use std::str::FromStr;

use super::entities::prelude::*;
use super::entities::*;
use chrono::{format::Fixed, FixedOffset};
use futures::{Future, StreamExt};
use rocket::{
    http::ext::IntoCollection,
    time::{format_description, macros::format_description, OffsetDateTime},
    tokio::spawn,
    State,
};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use twitter_v2::{Tweet, User};

pub async fn tweet(db: &State<DatabaseConnection>, tweet: &Tweet) -> () {
    let format = format_description::parse(
        "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour \
             sign:mandatory]:[offset_minute]",
    )
    .expect("Bad formatter");

    let date_string = &tweet
        .created_at
        .expect("Couldn't get the tweets date")
        .format(&format)
        .expect("Couldn't parse with thes formatter");

    println!("{date_string}");

    println!(
        "Foreign key is {}",
        tweet.author_id.expect("Bad author id").as_u64()
    );

    let converted_offset_date =
        chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&date_string)
            .expect("failed to parse date from string");

    let to_write = tweets::ActiveModel {
        id: ActiveValue::set(tweet.id.as_u64().try_into().expect("Bad tweet id")),
        conversation_id: ActiveValue::set(
            tweet
                .conversation_id
                .expect("Bad conversation id")
                .as_u64()
                .try_into()
                .expect("Failed to parse i64 from u64"),
        ),
        content: ActiveValue::set(tweet.text.clone()),
        author_id: ActiveValue::set(
            tweet
                .author_id
                .expect("Bad author id")
                .as_u64()
                .try_into()
                .expect("Failed to parse i64 from u64"),
        ),
        created_at: ActiveValue::set(converted_offset_date),
    };
    println!(
        "Seeing what the foreign key is:{}",
        to_write.author_id.clone().take().unwrap()
    );
    let res = Tweets::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert tweet into database");
}

pub async fn tweets(db: &State<DatabaseConnection>, tweets: &Vec<Tweet>) {
    let tweet_stream = futures::stream::iter(tweets.iter());
    tweet_stream.for_each(|t| tweet(db, t)).await;
}
