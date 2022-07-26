use crate::{app::load_user_from_twitter_handle, utils::TweetReferenceData};

use super::entities::prelude::*;
use super::entities::*;
use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use twitter_v2::{id::NumericId, Tweet, User};

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

pub async fn tweet_reference_by_id(
    db: &State<DatabaseConnection>,
    id: i64,
) -> Option<TweetReferenceData> {
    let db = db as &DatabaseConnection;

    let tweet_reference = TweetReferences::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet");

    match tweet_reference {
        Some(tweet_reference) => Some(tweet_reference.to_tweet_reference_data()),
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

pub async fn conversation(db: &State<DatabaseConnection>, conversation_id: i64) -> Vec<Tweet> {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::ConversationId.eq(conversation_id))
        .order_by_asc(tweets::Column::CreatedAt)
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
        .order_by_desc(tweets::Column::CreatedAt)
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

pub async fn does_tweet_exist(db: &State<DatabaseConnection>, id: i64) -> bool {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::Id.eq(id))
        .all(db)
        .await
        .expect("Failed to get tweet {id}")
        .len()
        == 1
}

pub async fn latest_tweet_from_user(db: &State<DatabaseConnection>, id: i64) -> Option<Tweet> {
    let db = db as &DatabaseConnection;

    let res = Tweets::find()
        .filter(tweets::Column::AuthorId.eq(id))
        .order_by_desc(tweets::Column::CreatedAt)
        .one(db)
        .await
        .expect("Failed to get tweet model");

    match res {
        Some(tweet_model) => Some(tweet_model.to_tweet()),
        None => None,
    }
}
