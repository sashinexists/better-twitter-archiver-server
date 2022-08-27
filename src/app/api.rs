use std::time::Duration;

use futures::future::join_all;
use rocket::time::OffsetDateTime;
use twitter_v2::authorization::BearerToken;
use twitter_v2::query::{TweetField, UserField};
use twitter_v2::{Tweet, TwitterApi, User};

use crate::utils::{TweetData, UserData};
pub async fn get_tweets_from_user(user: &User) -> Vec<TweetData> {
    let twitter_handle = &user.username;
    let api_tweets:Vec<Tweet> =load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(100)
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
        ])
        .send()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get @{twitter_handle}'s tweets from twitter api. \n\nError: {:?}",
                error
            )
        })
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open @{twitter_handle}'s tweets after fetching them from the server.")
        });
    join_all(api_tweets.into_iter().map(|api_tweet|TweetData::from_api_tweet(Some(api_tweet)))).await
}

pub async fn get_latest_tweet_from_user(user: &User) -> TweetData {
    let twitter_handle = &user.username;
    let api_tweet =load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(5)
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
        ])
        .send()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "User @{twitter_handle}'s tweets not loading. \n\nError: {:?}",
                error
            )
        })
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open @{twitter_handle}'s tweets after fetching them from the server.")
        })
        .first()
        .unwrap_or_else(|| panic!("Failed to get @{twitter_handle}'s latest tweet."))
        .clone();
    TweetData::from_api_tweet(Some(api_tweet)).await
}

pub async fn get_new_tweets_from_user(user: &User, from: &OffsetDateTime) -> Vec<TweetData> {
    let twitter_handle = &user.username;
    let api_tweets:Vec<Tweet> =load_api()
        .await
        .get_user_tweets(user.id)
        .start_time(*from)
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
        ])
        .send()
        .await
        .unwrap_or_else(|error|panic!("Failed to get @{twitter_handle}'s tweets. \n\nError: {:?}", error))
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open @{twitter_handle}'s new tweets after fetching them from the server.")
        });
    join_all(api_tweets.into_iter().map(|api_tweet|TweetData::from_api_tweet(Some(api_tweet)))).await
}


pub async fn get_first_hundred_tweets_from_user(user: &User) -> Vec<TweetData> {
    let twitter_handle = &user.username;
    let api_tweets:Vec<Tweet> =load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(100) //this line gets the max results
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
        ])
        .send()
        .await
        .unwrap_or_else(|error| panic!("Failed to load @{twitter_handle}'s first hundred tweets from the twitter api. \n\nError: {:?}", error))
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open @{twitter_handle}'s first hunrdred tweets after fetching them from the twitter api.")
        });
    join_all(api_tweets.into_iter().map(|api_tweet|TweetData::from_api_tweet(Some(api_tweet)))).await
}

pub async fn get_tweets_from_user_until_id(user: &User, id: u64) -> Vec<TweetData> {
    let twitter_handle = &user.username;
    let api_tweets:Vec<Tweet> =load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(100) 
        .until_id(id)
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
        ])
        .send()
        .await
        .unwrap_or_else(|error| panic!("Failed to get this batch of @{twitter_handle}'s tweets from the twitter api. \n\nError: {:?}", error))
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open this batch of @{twitter_handle}'s tweets after fetching them from the twitter api.")
        });
    join_all(api_tweets.into_iter().map(|api_tweet|TweetData::from_api_tweet(Some(api_tweet)))).await
}

pub async fn get_tweet_by_id(id: u64) -> TweetData {

    let api_tweet = match load_api()
        .await
        .get_tweet(id)
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::ConversationId,
            TweetField::AuthorId,
            TweetField::CreatedAt,
        ])
        .send()
        .await{
            Ok(tweet_response) => tweet_response.into_data(),
            Err(error) => {

            println!("Failed to get tweet of id {id} from the twitter api. \n\nError: {:?}\n\nWaiting 15 minutes and trying again...", error);
            tokio::time::sleep(Duration::from_secs(910)).await;
            println!("Finished waiting!");
            load_api()
                .await
                .get_tweet(id)
                .tweet_fields([
                    TweetField::Attachments,
                    TweetField::ReferencedTweets,
                    TweetField::ConversationId,
                    TweetField::AuthorId,
                    TweetField::CreatedAt,
                ])
                .send()
                .await
                .unwrap_or_else(|error|panic!("Second Attempt: Failed to get tweet of id {id} from the twitter api. \n\nError: {:?}", error))
                .into_data()
            }
        };

    TweetData::from_api_tweet(api_tweet).await

}


pub async fn get_user_by_twitter_handle(twitter_handle: &str) -> UserData {
    let api_user =
    load_api()
        .await
        .get_user_by_username(twitter_handle)
        .user_fields([UserField::Username, UserField::Description])
        .send()
        .await
        .unwrap_or_else(|error| panic!("Failed to get user @{twitter_handle} from the twitter api. \n\nError: {:?}", error))
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open @{twitter_handle}'s info after fetching it from the twitter api.")
        });
    UserData::from_api_user(&api_user).await
}

pub async fn get_user_by_id(id: u64) -> UserData {
    let api_user =
    load_api()
        .await
        .get_user(id)
        .user_fields([UserField::Username, UserField::Description])
        .send()
        .await
        .unwrap_or_else(|error| panic!("Failed to get user of id @{id} from the twitter api. \n\nError: {:?}", error))
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open user of id {id}'s info after fetching it from the twitter api.")
        });
    UserData::from_api_user(&api_user).await
}

pub async fn load_api() -> TwitterApi<BearerToken> {
    let auth = BearerToken::new(std::env::var("TWITTER_DEV_BEARER_TOKEN").unwrap());
    TwitterApi::new(auth)
}
