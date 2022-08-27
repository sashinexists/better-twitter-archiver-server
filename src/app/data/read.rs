use crate::{app::load_user_from_twitter_handle, utils::{TweetReferenceData, TweetData, UserData, ConversationData}};

use super::entities::prelude::*;
use super::entities::*;
use chrono::FixedOffset;
use futures::future::join_all;
use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use twitter_v2::{Tweet, User};

pub async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> TweetData {
    TweetData::read(db, id).await
}



pub async fn user_by_id(db: &State<DatabaseConnection>, id: i64) -> UserData {
    UserData::read(db, id).await
}

pub async fn user_by_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> UserData{
    UserData::read_from_twitter_handle(db, twitter_handle).await
}

pub async fn tweets(db: &State<DatabaseConnection>) -> Vec<TweetData> {
    let tweet_models:Vec<tweets::Model> = Tweets::find()
        .all(db as &DatabaseConnection)
        .await
        .unwrap_or_else(|error| panic!("Failed to get tweets from database. Error: {:?}", error));
    join_all(tweet_models.into_iter().map(|tweet_model|TweetData::read_from_data_model(db,tweet_model))).await
}

pub async fn conversation(db: &State<DatabaseConnection>, conversation_id: i64) -> ConversationData {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::ConversationId.eq(conversation_id))
        .order_by_asc(tweets::Column::CreatedAt)
        .all(db)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get conversations from database. Error: {:?}",
                error,
            )
        })
        .into_iter()
        .map(|tweet_model|TweetData::read_from_data_model(db, tweet_model) )
        .collect::<Vec<twitter_v2::Tweet>>()
}

pub async fn users(db: &State<DatabaseConnection>) -> Vec<UserData> {
    let db = db as &DatabaseConnection;

    Users::find()
        .all(db)
        .await
        .unwrap_or_else(|error| panic!("Failed to get users from database. Error: {:?}", error))
        .into_iter()
        .map(|b| b.to_twitter_user())
        .collect::<Vec<twitter_v2::User>>()
}

pub async fn users_tweets(db: &State<DatabaseConnection>, twitter_handle: &str) -> Vec<TweetData> {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    let username = user.name;

    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::AuthorId.eq(user.id.as_u64()))
        .order_by_desc(tweets::Column::CreatedAt)
        .all(db)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get @{username}'s tweets from database. Error: {:?}",
                error
            )
        })
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>()
}

pub async fn users_tweets_since_date(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
    rfc3339_date: &str,
) -> Vec<Tweet> {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    let username = user.name;

    let date =
        chrono::DateTime::<FixedOffset>::parse_from_rfc3339(rfc3339_date).unwrap_or_else(|error| {
            panic!(
                "Failed to parse date from rfc3339_date {:?}. Error: {:?}",
                rfc3339_date, error
            )
        });

    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::AuthorId.eq(user.id.as_u64()))
        .filter(tweets::Column::CreatedAt.gt(date))
        .order_by_desc(tweets::Column::CreatedAt)
        .all(db)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get @{username}'s tweets from the database. Error: {:?}",
                error
            )
        })
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
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get conversation {id} from the database. Error {:?}",
                error
            )
        })
        .len()
        == 1
}

pub async fn does_tweet_exist(db: &State<DatabaseConnection>, id: i64) -> bool {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::Id.eq(id))
        .all(db)
        .await
        .unwrap_or_else(|error| {
            panic!("Failed to read tweet {id} from database. Error {:?}", error)
        })
        .len()
        == 1
}

pub async fn latest_tweet_from_user(db: &State<DatabaseConnection>, id: i64) -> Option<Tweet> {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::AuthorId.eq(id))
        .order_by_desc(tweets::Column::CreatedAt)
        .one(db)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get tweet model for tweet {id} from database. Error: {:?}",
                error
            )
        })
        .map(|tweet_model| tweet_model.to_tweet())
}

pub async fn search_tweets_in_db(db: &State<DatabaseConnection>, search_query: &str) -> Vec<Tweet> {
    let db = db as &DatabaseConnection;

    Tweets::find()
        .filter(tweets::Column::Content.contains(search_query))
        .order_by_desc(tweets::Column::CreatedAt)
        .all(db)
        .await
        .expect("Failed to run tweet search")
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>()
}
