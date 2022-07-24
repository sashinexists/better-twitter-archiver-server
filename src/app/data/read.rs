use super::entities::prelude::*;
use super::entities::*;
use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> Option<String> {
    let db = db as &DatabaseConnection;

    let tweet = Tweets::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet");

    match tweet {
        Some(tweet) => Some(
            ron::ser::to_string_pretty(&tweet.to_tweet(), ron::ser::PrettyConfig::new())
                .expect("Failed to parse ron"),
        ),
        None => None,
    }
}

pub async fn user_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    let db = db as &DatabaseConnection;

    let user = Users::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet")
        .expect("Failed to open the option model tweet")
        .to_twitter_user();

    ron::ser::to_string_pretty(&user, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

pub async fn user_by_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> String {
    let db = db as &DatabaseConnection;

    let user = Users::find()
        .filter(users::Column::Username.eq(twitter_handle))
        .one(db)
        .await
        .expect("Failed to open the result option model tweet")
        .expect("Failed to open the option model tweet")
        .to_twitter_user();

    ron::ser::to_string_pretty(&user, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

pub async fn tweets(db: &State<DatabaseConnection>) -> String {
    let db = db as &DatabaseConnection;

    let tweets: Vec<twitter_v2::Tweet> = Tweets::find()
        .all(db)
        .await
        .expect("Failed to get tweets")
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>();

    ron::ser::to_string_pretty(&tweets, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

pub async fn users(db: &State<DatabaseConnection>) -> String {
    let db = db as &DatabaseConnection;

    let users: Vec<twitter_v2::User> = Users::find()
        .all(db)
        .await
        .expect("Failed to get users")
        .into_iter()
        .map(|b| b.to_twitter_user())
        .collect::<Vec<twitter_v2::User>>();

    ron::ser::to_string_pretty(&users, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

pub async fn users_tweets(db: &State<DatabaseConnection>, twitter_handle: &str) -> String {
    let db = db as &DatabaseConnection;

    let user = Users::find()
        .filter(users::Column::Username.eq(twitter_handle))
        .one(db)
        .await
        .expect("Failed to open the result option model tweet")
        .expect("Failed to open the option model tweet")
        .to_twitter_user();

    let username = user.name;

    let users_tweets = Tweets::find()
        .filter(tweets::Column::AuthorId.eq(user.id.as_u64()))
        .all(db)
        .await
        .expect(&format!("Failed to get @{username}'s tweets"))
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>();

    ron::ser::to_string_pretty(&users_tweets, ron::ser::PrettyConfig::new())
        .expect("Failed to parse ron")
}

/*
   you will also want get user by id, get conversation by id, get user's tweets,
   it will need to know when to update the tweets

*/
