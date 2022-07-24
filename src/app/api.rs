use twitter_v2::authorization::BearerToken;
use twitter_v2::data::ReferencedTweetKind::RepliedTo;
use twitter_v2::id::NumericId;
use twitter_v2::query::{TweetField, UserField};
use twitter_v2::{Tweet, TwitterApi, User};
/*
use async_recursion::async_recursion;

#[async_recursion]
pub async fn get_twitter_conversation_from_tweet(tweet: Tweet) -> Vec<Tweet> {
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
                let replied_to: Tweet = super::load_tweet_from_id(replied_to_id).await;
                let mut conversation: Vec<Tweet> =
                    get_twitter_conversation_from_tweet(replied_to).await;
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
}*/

pub async fn get_tweets_from_user(user: &User) -> Vec<Tweet> {
    load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(10) //this line gets the max results
        .tweet_fields([
            TweetField::Attachments,
            TweetField::ReferencedTweets,
            TweetField::AuthorId,
            TweetField::ConversationId,
            TweetField::CreatedAt,
        ])
        .send()
        .await
        .expect("Users tweets not loading")
        .into_data()
        .expect("Failure to open option<Vec<Tweet>>")
}

pub async fn get_all_tweets_from_user(user: &User) -> Vec<Tweet> {
    let mut output = get_first_hundred_tweets_from_user(user).await;
    const FIRST_TWEET_ID: u64 = 1012187366587392000; // 1490542591154130947; //@yudapearls first tweet id = 1012187366587392000
    let mut last_id = output.last().expect("Failed to get last tweet").id.as_u64();
    let mut i = 1;
    while i < 32 {
        output.append(&mut get_tweets_from_user_until_id(user, last_id).await);
        last_id = output.last().expect("Failed to get last tweet").id.as_u64();
        println!("Loading tweets up to {i}00");
        i += 1;
    }

    output
}

pub async fn get_first_hundred_tweets_from_user(user: &User) -> Vec<Tweet> {
    load_api()
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
        .expect("Users tweets not loading")
        .into_data()
        .expect("Failure to open option<Vec<Tweet>>")
}

pub async fn get_tweets_from_user_until_id(user: &User, id: u64) -> Vec<Tweet> {
    load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(100) //this line gets the max results
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
        .expect("Users tweets not loading")
        .into_data()
        .expect("Failure to open option<Vec<Tweet>>")
}

pub async fn get_tweet_by_id(id: u64) -> Tweet {
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
        .expect("this tweet should exist")
        .into_data()
        .expect("Failure to open Option<Tweet>")
}

pub async fn get_user_by_twitter_handle(twitter_handle: &str) -> User {
    load_api()
        .await
        .get_user_by_username(twitter_handle)
        .user_fields([UserField::Username, UserField::Description])
        .send()
        .await
        .expect("This user should exist")
        .into_data()
        .expect("Failure to open Option<User>")
}

pub async fn get_user_by_id(id: u64) -> User {
    load_api()
        .await
        .get_user(id)
        .user_fields([UserField::Username, UserField::Description])
        .send()
        .await
        .expect("This user should exist")
        .into_data()
        .expect("Failure to open Option<User>")
}

pub async fn load_api() -> TwitterApi<BearerToken> {
    let auth = BearerToken::new(std::env::var("TWITTER_DEV_BEARER_TOKEN").unwrap());
    TwitterApi::new(auth)
}
