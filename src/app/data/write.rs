use crate::app::{load_tweet_from_id, load_user_from_id};

use super::super::super::utils::{convert_date_to_chrono, TweetReferenceData};
use super::entities::prelude::*;
use super::entities::*;
use chrono::{format::Fixed, DateTime, FixedOffset};
use futures::{Future, StreamExt};
use rocket::{
    http::ext::IntoCollection,
    time::{format_description, macros::format_description, OffsetDateTime},
    tokio::spawn,
    State,
};
use std::str::FromStr;

use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use twitter_v2::{Tweet, User};

pub async fn tweet(db: &State<DatabaseConnection>, tweet: &Tweet) -> () {
    let author_id = tweet
        .author_id
        .expect("Bad author id")
        .as_u64()
        .try_into()
        .expect("Failed to parse i64 from u64");

    let conversation_id = tweet
        .conversation_id
        .expect("Bad conversation id")
        .clone()
        .as_u64()
        .try_into()
        .expect("Failed to parse i64 from u64");

    load_user_from_id(&db, author_id).await;
    if !super::read::does_conversation_exist(db, conversation_id).await {
        conversation(db, &conversation_id).await;
    }

    let converted_offset_date = convert_date_to_chrono(tweet.created_at);

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

    let res = Tweets::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert tweet into database");
}

pub async fn tweets(db: &State<DatabaseConnection>, tweets: &Vec<Tweet>) {
    let tweet_stream = futures::stream::iter(tweets.iter());
    tweet_stream.for_each(|t| tweet(db, t)).await;
}

pub async fn user(db: &State<DatabaseConnection>, user: &User) -> () {
    let to_write = users::ActiveModel {
        id: ActiveValue::Set(user.id.as_u64().try_into().unwrap()),
        name: ActiveValue::Set(user.name.clone()),
        username: ActiveValue::Set(user.username.clone()),
        description: ActiveValue::Set(
            user.description
                .clone()
                .expect("Failed to unwrap description"),
        ),
    };
    let res = Users::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert user into database");
}

pub async fn conversation(db: &State<DatabaseConnection>, conversation_id: &i64) -> () {
    let to_write = conversations::ActiveModel {
        id: ActiveValue::Set(conversation_id.clone()),
    };
    let res = Conversations::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert conversation {conversation_id} into database");
}

pub async fn tweet_reference(
    db: &State<DatabaseConnection>,
    tweet_reference_data: &TweetReferenceData,
) -> () {
    let referenced_tweet_id = tweet_reference_data.reference_tweet_id;

    if !super::read::does_tweet_exist(db, referenced_tweet_id).await {
        load_tweet_from_id(db, referenced_tweet_id).await;
    }

    let to_write = tweet_references::ActiveModel {
        source_tweet_id: ActiveValue::Set(tweet_reference_data.source_tweet_id.clone()),
        reference_type: ActiveValue::Set(tweet_reference_data.type_to_string()),
        referenced_tweet_id: ActiveValue::Set(tweet_reference_data.reference_tweet_id),
    };
    let res = TweetReferences::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert conversation {conversation_id} into database");
}
