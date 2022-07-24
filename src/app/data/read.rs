use crate::app::load_user_from_twitter_handle;

use super::entities::prelude::*;
use super::entities::*;
use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use twitter_v2::{Tweet, User};

pub async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> Option<Tweet> {
    let db = db as &DatabaseConnection;

    let tweet = Tweets::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet");

    match tweet {
        Some(tweet) => Some(tweet.to_tweet()),
        None => None,
    }
}

pub async fn user_by_id(db: &State<DatabaseConnection>, id: i64) -> Option<User> {
    let db = db as &DatabaseConnection;

    let user = Users::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet");

    match user {
        Some(user) => Some(user.to_twitter_user()),
        None => None,
    }
}

pub async fn user_by_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Option<User> {
    let db = db as &DatabaseConnection;

    let user = Users::find()
        .filter(users::Column::Username.eq(twitter_handle))
        .one(db)
        .await
        .expect("Failed to open the result option model tweet");

    match user {
        Some(user) => Some(user.to_twitter_user()),
        None => None,
    }
}

pub async fn tweets(db: &State<DatabaseConnection>) -> Vec<Tweet> {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .all(db)
        .await
        .expect("Failed to get tweets")
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>()
}

pub async fn users(db: &State<DatabaseConnection>) -> Vec<User> {
    let db = db as &DatabaseConnection;

    Users::find()
        .all(db)
        .await
        .expect("Failed to get users")
        .into_iter()
        .map(|b| b.to_twitter_user())
        .collect::<Vec<twitter_v2::User>>()
}

pub async fn users_tweets(db: &State<DatabaseConnection>, twitter_handle: &str) -> Vec<Tweet> {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    let username = user.name;

    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::AuthorId.eq(user.id.as_u64()))
        .all(db)
        .await
        .expect(&format!("Failed to get @{username}'s tweets"))
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>()
}

pub async fn does_conversation_exist(db: &State<DatabaseConnection>, id: i64) -> bool {
    let db = db as &DatabaseConnection;

    Conversations::find()
        .filter(conversations::Column::Id.eq(id))
        .all(db)
        .await
        .expect("Failed to get conversation {id}")
        .len()
        == 1
}

/*
   you will also want get user by id, get conversation by id, get user's tweets,
   it will need to know when to update the tweets

*/
