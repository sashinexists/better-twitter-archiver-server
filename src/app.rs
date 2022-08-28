use crate::utils::{i64_to_u64, u64_to_i64, TweetData, TweetReferenceData, UserData};
use rocket::{time::OffsetDateTime, State};
use sea_orm::DatabaseConnection;
use std::collections::VecDeque;
use twitter_v2::{data::ReferencedTweetKind::RepliedTo, Tweet, User};
pub mod api;
pub mod data;

pub async fn load_tweet_from_id(db: &State<DatabaseConnection>, id: i64) -> TweetData {
    let tweet_data = TweetData::read(db, id).await;
    let tweet = tweet_data.tweet.clone();
    match tweet {
        Some(_tweet) => tweet_data,
        None => {
            let tweet_data = api::get_tweet_by_id(i64_to_u64(id)).await;
            let tweet = tweet_data.tweet.clone();
            match tweet {
                Some(_tweet) => {
                    tweet_data.write(db);
                    tweet_data
                },
                None => TweetData::empty() 
            }
        }
    }
}

pub async fn load_user_from_id(db: &State<DatabaseConnection>, id: i64) -> UserData {
    let user_data = UserData::read(db, id).await;
    let user = user_data.user.clone();
    match user {
        Some(_user)=>user_data,
        None=> {
            let user_data = api::get_user_by_id(i64_to_u64(id)).await;
            let user = user_data.user.clone();
            match user {
                Some(_user) => {
                    user_data.write(db);
                    user_data
                },
                None => UserData::empty().await
            }
        }
    }
}

pub async fn load_user_from_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> UserData {
    let user_data = UserData::read_from_twitter_handle(db, twitter_handle).await;
    let user = user_data.user.clone();
    match user {
        Some(_user)=>user_data,
        None=> {
            let user_data = api::get_user_by_twitter_handle(twitter_handle).await;
            let user = user_data.user.clone();
            match user {
                Some(_user) => {
                    user_data.write(db);
                    user_data
                },
                None => UserData::empty().await
            }
        }
    }
}

pub async fn load_user_tweets_from_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Vec<TweetData> {
    let user_tweets = data::read::users_tweets(db, twitter_handle).await;
    //need something better
    if user_tweets.is_empty() {
        let user_tweets =
            api::get_tweets_from_user(&load_user_from_twitter_handle(db, twitter_handle).await)
                .await;
        data::write::tweets(db, &user_tweets).await;
        user_tweets
    } else if has_new_tweets(db, twitter_handle).await {
        println!("Adding new tweets");
        let new_tweets = load_users_new_tweets(db, twitter_handle).await;
        data::write::tweets(db, &new_tweets).await;
        data::read::users_tweets(db, twitter_handle).await
    } else {
        println!("No new tweets to add");
        user_tweets
    }
}

pub async fn load_user_conversations_from_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Vec<ConversationData> {
    let users_tweets = load_user_tweets_from_twitter_handle(db, twitter_handle).await;
    let mut output: Vec<Vec<Tweet>> = Vec::<Vec<Tweet>>::new();
    for (i, tweet) in users_tweets.iter().enumerate().take(400) {
        let tweet_id = &tweet.id.as_u64();
        println!("\nLoading conversation {i} from tweet of id {tweet_id}\n");
        output.push(
            load_twitter_conversation_from_tweet_id(db, tweet.id.as_u64().try_into().unwrap())
                .await,
        );
    }
    output
}

pub async fn seed_user_tweets_from_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Vec<TweetData> {
    let user_tweets =
        api::get_tweets_from_user(&load_user_from_twitter_handle(db, twitter_handle).await)
            .await;
    data::write::tweets(db, &user_tweets).await;
    data::read::users_tweets(db, twitter_handle).await
}

pub async fn load_offset_datetime_for_users_latest_tweet_in_database(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> OffsetDateTime {
    let user_data = load_user_from_twitter_handle(db, twitter_handle).await;
    let user = user_data.user.clone().unwrap_or_else(||panic!("User @{twitter_handle} is None"));
    let user_id: i64 = user
        .id;
    data::read::latest_tweet_from_user(db, user_id)
        .await
        .tweet
        .unwrap_or_else(||panic!("Invalid tweet"))
        .created_at
}

pub async fn load_offset_datetime_for_users_latest_tweet(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> OffsetDateTime {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    api::get_latest_tweet_from_user(&user)
        .await
        .created_at
        .expect("Failed to get the offsetdatetime for latest tweet")
}

pub async fn has_new_tweets(db: &State<DatabaseConnection>, twitter_handle: &str) -> bool {
    let latest_db_tweet_date =
        load_offset_datetime_for_users_latest_tweet_in_database(db, twitter_handle).await;
    let latest_tweet_date = load_offset_datetime_for_users_latest_tweet(db, twitter_handle).await;
    let difference = latest_tweet_date.unix_timestamp() - latest_db_tweet_date.unix_timestamp();
    difference > 0
}

pub async fn has_user_tweeted_since_date(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
    date_unix_timestamp: i64,
) -> bool {
    let latest_tweet_date = load_offset_datetime_for_users_latest_tweet(db, twitter_handle).await;
    let difference = latest_tweet_date.unix_timestamp() - date_unix_timestamp;
    difference > 0
}

pub async fn load_users_tweets_since_date(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
    rfc3339_date: &str,
) -> Vec<TweetData> {
    data::write::tweets(db, &load_users_new_tweets(db, twitter_handle).await).await;
    data::read::users_tweets_since_date(db, twitter_handle, rfc3339_date).await
}

pub async fn load_users_new_tweets(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Vec<TweetData> {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    let from = load_offset_datetime_for_users_latest_tweet_in_database(db, twitter_handle).await;
    api::get_new_tweets_from_user(&user, &from).await
}
pub async fn load_twitter_conversation_from_tweet_id(
    db: &State<DatabaseConnection>,
    tweet_id: i64,
) -> Vec<TweetData> {
    let tweet = load_tweet_from_id(db, tweet_id).await;
    let mut conversation: VecDeque<Tweet> = VecDeque::from(vec![tweet.clone()])
        .into_iter()
        .flatten()
        .collect();
    let mut reference_tweets = conversation[0].clone().referenced_tweets;
    println!("The tweets is {:?}", &tweet);
    println!("The referenced_tweets are {:?}", reference_tweets);
    while reference_tweets.is_some()
        && reference_tweets
            .clone()
            .unwrap()
            .iter()
            .any(|reference| reference.kind == RepliedTo)
    {
        let references = reference_tweets.unwrap();
        let replied_to_id: i64 = u64_to_i64(
            references
                .iter()
                .find(|reference| reference.kind == RepliedTo)
                .expect("Failed to find the replied to tweet")
                .id
                .as_u64(),
        );
        conversation.push_front(
            load_tweet_from_id(db, replied_to_id)
                .await
                .expect("Failed to load replied to tweet of id {replied_to_id}."),
        );
        data::write::tweet_reference(
            db,
            TweetReferenceData {
                reference_type: RepliedTo,
                source_tweet_id: tweet_id,
                reference_tweet_id: replied_to_id,
            },
        );
        reference_tweets = conversation[0].clone().referenced_tweets;
    }
    println!("This conversation is {} tweet(s) long.", conversation.len());
    Vec::from(conversation)
}
pub async fn search_tweets_in_db(db: &State<DatabaseConnection>, search_query: &str) -> Vec<TweetData> {
    data::read::search_tweets_in_db(db, search_query).await
}
