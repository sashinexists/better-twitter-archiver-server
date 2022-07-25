use rocket::State;
use sea_orm::DatabaseConnection;
use twitter_v2::{Tweet, User};

use crate::utils::TweetReferenceData;

use self::data::entities::tweets;

pub mod api;
pub mod data;

pub async fn load_tweet_from_id(db: &State<DatabaseConnection>, id: i64) -> Tweet {
    match data::read::tweet_by_id(db, id).await {
        Some(tweet) => tweet,
        None => {
            let tweet = api::get_tweet_by_id(id.try_into().unwrap()).await;
            data::write::tweet(db, &tweet).await;
            tweet
        }
    }
}

//unsure about this one
pub async fn load_tweet_reference(
    db: &State<DatabaseConnection>,
    id: i64,
) -> Option<TweetReferenceData> {
    data::read::tweet_reference_by_id(db, id).await
}

pub async fn load_user_from_id(db: &State<DatabaseConnection>, id: i64) -> User {
    match data::read::user_by_id(db, id).await {
        Some(user) => user,
        None => {
            let user = api::get_user_by_id(id.try_into().unwrap()).await;
            data::write::user(db, &user).await;
            user
        }
    }
}

pub async fn load_user_from_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> User {
    match data::read::user_by_twitter_handle(db, &twitter_handle).await {
        Some(user) => user,
        None => {
            let user = api::get_user_by_twitter_handle(&twitter_handle).await;
            data::write::user(db, &user).await;
            user
        }
    }
}

pub async fn load_user_tweets_from_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Vec<Tweet> {
    let user_tweets = data::read::users_tweets(db, twitter_handle).await;
    if user_tweets.len() == 0 {
        let user_tweets =
            api::get_all_tweets_from_user(&load_user_from_twitter_handle(db, twitter_handle).await)
                .await;
        data::write::tweets(db, &user_tweets).await;
        user_tweets
    } else {
        user_tweets
    }
}
