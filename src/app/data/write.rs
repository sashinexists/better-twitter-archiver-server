use super::super::super::utils::{convert_date_to_chrono, TweetReferenceData};
use super::entities::prelude::*;
use super::entities::*;
use crate::app::{load_tweet_from_id, load_user_from_id};
use async_recursion::async_recursion;
use futures::StreamExt;
use rocket::State;
use twitter_v2::data::ReferencedTweet;

use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use twitter_v2::{Tweet, User};

pub async fn tweet(db: &State<DatabaseConnection>, tweet: &Tweet) {
    let author_id = tweet
        .author_id
        .unwrap_or_else(||panic!("Couldn't get author_id from tweet. \n\nTweet: {:?}",tweet))
        .as_u64()
        .try_into()
        .unwrap_or_else(|error|panic!("Failed to parse author id from u64 to i64. \n\nTweet: {:?}\n\nError: {:?}\n", tweet, error));

    let conversation_id = tweet
        .conversation_id
        .unwrap_or_else(||panic!("Couldn't get conversation id from tweet. \n\nTweet: {:?}",tweet))
        .as_u64()
        .try_into()
        .unwrap_or_else(|error|panic!("Failed to parse conversation id from u64 to i64. \n\nTweet: {:?}\n\nError: {:?}\n", tweet, error));

    load_user_from_id(db, author_id).await;
    if !super::read::does_conversation_exist(db, conversation_id).await {
        conversation(db, &conversation_id).await;
    }

    let converted_offset_date = convert_date_to_chrono(tweet.created_at);

    let to_write = tweets::ActiveModel {
        id: ActiveValue::set(tweet.id.as_u64().try_into().expect("Bad tweet id")),
        conversation_id: ActiveValue::set(conversation_id),
        content: ActiveValue::set(tweet.text.clone()),
        author_id: ActiveValue::set(author_id),
        created_at: ActiveValue::set(converted_offset_date),
    };

    let res = Tweets::insert(to_write).exec(db.inner()).await;

    match res {
        Ok(_res) => (),
        Err(e) => println!(
            "Failed to to write tweet {} to the database because {}",
            tweet.id, e
        ),
    }
}

pub async fn tweet_with_reference(db: &State<DatabaseConnection>, tweet: &Tweet) {
    let tweet_id: i64 = tweet
        .id
        .as_u64()
        .try_into()
        .unwrap_or_else(|error|panic!("Failed to parse tweet id from u64 to i64. \n\nTweet: {:?}\n\nError: {:?}\n", tweet, error));

    let author_id = tweet
        .author_id
        .expect("Bad author id")
        .as_u64()
        .try_into()
        .unwrap_or_else(|error|panic!("Failed to parse author id from u64 to i64. \n\nTweet: {:?}\n\nError: {:?}\n", tweet, error));

    let conversation_id = tweet
        .conversation_id
        .expect("Bad conversation id")
        .as_u64()
        .try_into()
        .unwrap_or_else(|error|panic!("Failed to parse conversation id from u64 to i64. \n\nTweet: {:?}\n\nError: {:?}\n", tweet, error));


    load_user_from_id(db, author_id).await;
    if !super::read::does_conversation_exist(db, conversation_id).await {
        conversation(db, &conversation_id).await;
    }

    let converted_offset_date = convert_date_to_chrono(tweet.created_at);

    let to_write = tweets::ActiveModel {
        id: ActiveValue::set(tweet_id),
        conversation_id: ActiveValue::set(conversation_id
        ),
        content: ActiveValue::set(tweet.text.clone()),
        author_id: ActiveValue::set(author_id
        ),
        created_at: ActiveValue::set(converted_offset_date),
    };

    let res = Tweets::insert(to_write).exec(db.inner()).await;

    match res {
        Ok(_res) => (),
        Err(e) => println!(
            "Failed to to write tweet {} to the database because {}",
            tweet.id, e
        ),
    }

    let referenced_tweets = tweet.referenced_tweets.clone();

    match referenced_tweets {
        Some(references) => tweet_references(db, tweet_id, references).await,
        None => println!("No referenced tweets"),
    }
}

pub async fn tweets(db: &State<DatabaseConnection>, tweets: &[Tweet]) {
    let tweet_stream = futures::stream::iter(tweets.iter());
    tweet_stream.for_each(|t| tweet(db, t)).await;
}

pub async fn user(db: &State<DatabaseConnection>, user: &User) {
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
    let _res = Users::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert user into database");
}

pub async fn conversation(db: &State<DatabaseConnection>, conversation_id: &i64) {
    let to_write = conversations::ActiveModel {
        id: ActiveValue::Set(*conversation_id),
    };
    let _res = Conversations::insert(to_write)
        .exec(db.inner())
        .await
        .expect("failed to insert conversation {conversation_id} into database");
}

#[async_recursion]
pub async fn tweet_reference(
    db: &State<DatabaseConnection>,
    tweet_reference_data: TweetReferenceData,
) -> () {
    let referenced_tweet_id = tweet_reference_data.reference_tweet_id;
    let source_tweet_id = tweet_reference_data.source_tweet_id;
    if !super::read::does_tweet_exist(db, source_tweet_id).await {
        load_tweet_from_id(db, source_tweet_id).await;
    }

    if !super::read::does_tweet_exist(db, referenced_tweet_id).await {
        load_tweet_from_id(db, referenced_tweet_id).await;
    }

    let to_write = tweet_references::ActiveModel {
        source_tweet_id: ActiveValue::Set(tweet_reference_data.source_tweet_id),
        reference_type: ActiveValue::Set(tweet_reference_data.type_to_string()),
        referenced_tweet_id: ActiveValue::Set(referenced_tweet_id),
    };
    let res = TweetReferences::insert(to_write).exec(db.inner()).await;

    match res {
        Ok(_res) => (),
        Err(e) => println!(
            "Failed to add tweet reference {} to the database because of {}",
            referenced_tweet_id, e
        ),
    }
}

pub async fn tweet_references(
    db: &State<DatabaseConnection>,
    tweet_id: i64,
    tweet_references: Vec<ReferencedTweet>,
) {
    let tweet_reference_stream = futures::stream::iter(tweet_references.iter());
    tweet_reference_stream
        .for_each(|tweet_ref| async {
            let tweet_reference_data =
                TweetReferenceData::from_referenced_tweet(tweet_id, tweet_ref);

            tweet_reference(db, tweet_reference_data).await
        })
        .await;
}
