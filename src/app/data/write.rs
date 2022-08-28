use super::entities::prelude::*;
use super::entities::*;
use crate::app::load_user_from_id;
use crate::utils::{TweetData, UserData};
use futures::StreamExt;
use rocket::State;

use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

pub async fn tweet(db: &State<DatabaseConnection>, tweet_data: &TweetData) {
    let tweet = tweet_data.tweet.clone();
    if let Some(tweet) = tweet {
    load_user_from_id(db, tweet.author_id).await;
    if !super::read::does_conversation_exist(db, tweet.conversation_id).await {
        conversation(db, &tweet.conversation_id).await;
    }
    }

    tweet_data.write(db);
}

pub async fn tweets(db: &State<DatabaseConnection>, tweets: &[TweetData]) {
    let tweet_stream = futures::stream::iter(tweets.iter());
    tweet_stream.for_each(|t| tweet(db, t)).await;
}

pub async fn user(db: &State<DatabaseConnection>, user: &UserData) {
   user.write(db); 
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
