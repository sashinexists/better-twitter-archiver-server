use super::entities::prelude::*;
use super::entities::*;
use rocket::State;
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    let db = db as &DatabaseConnection;

    let tweet = Tweets::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet")
        .expect("Failed to open the option model tweet")
        .to_tweet();

    ron::ser::to_string_pretty(&tweet, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
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

pub async fn tweets(db: &State<DatabaseConnection>) -> String {
    let db = db as &DatabaseConnection;

    let tweets: Vec<twitter_v2::Tweet> = Tweets::find()
        .all(db)
        .await
        .unwrap()
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
        .unwrap()
        .into_iter()
        .map(|b| b.to_twitter_user())
        .collect::<Vec<twitter_v2::User>>();

    ron::ser::to_string_pretty(&users, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

/*
   you will also want get user by id, get conversation by id, get user's tweets,
   it will need to know when to update the tweets

*/
