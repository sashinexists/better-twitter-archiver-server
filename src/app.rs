use crate::utils::TweetReferenceData;
use async_recursion::async_recursion;
use futures::{executor::block_on, StreamExt};
use rocket::{http::ext::IntoCollection, time::OffsetDateTime, State};
use sea_orm::DatabaseConnection;
use twitter_v2::{data::ReferencedTweetKind::RepliedTo, Tweet, User};

pub mod api;
pub mod data;

pub async fn load_tweet_from_id(db: &State<DatabaseConnection>, id: i64) -> Option<Tweet> {
    match data::read::tweet_by_id(db, id).await {
        Some(tweet) => Some(tweet),
        None => match api::get_tweet_by_id(id.try_into().unwrap_or_else(|error| {
            panic!(
                "Failed to parse tweet id {id} from i64 to u64. \n\nError: {:?}",
                error
            )
        }))
        .await
        {
            Some(tweet) => {
                data::write::tweet(db, &tweet).await;
                Some(tweet)
            }
            None => {
                println!("Failed to load tweet from id {id}");
                None
            }
        },
    }
}

pub async fn load_tweet_with_reference_from_id(
    db: &State<DatabaseConnection>,
    id: i64,
) -> Option<Tweet> {
    let tweet = api::get_tweet_by_id(id.try_into().unwrap()).await;
    match tweet {
        Some(tweet) => {
            data::write::tweet_with_reference(db, &tweet).await;
            Some(tweet)
        }
        None => {
            println!("Couldn't load tweet from id {id}. It was probably deleted.");
            None
        }
    }
}

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
    match data::read::user_by_twitter_handle(db, twitter_handle).await {
        Some(user) => user,
        None => {
            let user = api::get_user_by_twitter_handle(twitter_handle).await;
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
) -> Vec<Vec<Tweet>> {
    let users_tweets = load_user_tweets_from_twitter_handle(db, twitter_handle).await;
    let mut output: Vec<Vec<Tweet>> = Vec::<Vec<Tweet>>::new();
    for (i, tweet) in users_tweets.iter().enumerate().take(30) {
        let tweet_id = &tweet.id.as_u64();
        println!("Loading conversation {i} from tweet of id {tweet_id}");
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
) -> Vec<Tweet> {
    let user_tweets =
        api::get_all_tweets_from_user(&load_user_from_twitter_handle(db, twitter_handle).await)
            .await;
    data::write::tweets(db, &user_tweets).await;
    data::read::users_tweets(db, twitter_handle).await
}

pub async fn load_offset_datetime_for_users_latest_tweet_in_database(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> OffsetDateTime {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    let user_id: i64 = user
        .id
        .as_u64()
        .try_into()
        .expect("Failed to convert u64 to i64");
    data::read::latest_tweet_from_user(db, user_id)
        .await
        .expect("failed to get recent tweet")
        .created_at
        .expect("Failed to get offset datetime of the most recent tweet")
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
) -> Vec<Tweet> {
    data::write::tweets(db, &load_users_new_tweets(db, twitter_handle).await).await;
    data::read::users_tweets_since_date(db, twitter_handle, rfc3339_date).await
}

pub async fn load_users_new_tweets(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> Vec<Tweet> {
    let user = load_user_from_twitter_handle(db, twitter_handle).await;
    let from = load_offset_datetime_for_users_latest_tweet_in_database(db, twitter_handle).await;
    api::get_new_tweets_from_user(&user, &from).await
}

// pub async fn load_twitter_conversation_from_tweet_id(
//     db: &State<DatabaseConnection>,
//     tweet_id: i64,
// ) -> Vec<Tweet> {
//     let tweet = load_tweet_with_reference_from_id(db, tweet_id).await;
//     match tweet {
//         Some(tweet) => {
//             data::read::conversation(
//                 db,
//                 tweet
//                     .conversation_id
//                     .expect("bad conversation id")
//                     .as_u64()
//                     .try_into()
//                     .unwrap(),
//             )
//             .await
//         }
//         None => Vec::<Tweet>::new(),
//     }
// }
//

#[async_recursion]
pub async fn load_twitter_conversation_from_tweet_id(
    db: &State<DatabaseConnection>,
    tweet_id: i64,
) -> Vec<Tweet> {
    let tweet = load_tweet_with_reference_from_id(db, tweet_id)
        .await
        .unwrap_or_else(|| panic!("Couldn't load tweet"));
    let mut output = vec![tweet];
    match &output[0].referenced_tweets {
        Some(referenced_tweets) => {
            if referenced_tweets
                .iter()
                .any(|tweet| tweet.kind == RepliedTo)
            {
                let replied_to_id = referenced_tweets
                    .iter()
                    .find(|tweet| tweet.kind == RepliedTo)
                    .expect("Failed to find replied to tweet")
                    .id
                    .as_u64();
                let mut conversation: Vec<Tweet> = load_twitter_conversation_from_tweet_id(
                    db,
                    replied_to_id.try_into().unwrap_or_else(|error| {
                        panic!(
                            "Failed to parse {replied_to_id} from u64 to i64. \n\nError: {:?}",
                            error
                        )
                    }),
                )
                .await;
                output.append(&mut conversation);
                output
            } else {
                output.reverse();
                output
            }
        }
        None => {
            output.reverse();
            output
        }
    }
}

pub async fn search_tweets_in_db(db: &State<DatabaseConnection>, search_query: &str) -> Vec<Tweet> {
    data::read::search_tweets_in_db(db, search_query).await
}
