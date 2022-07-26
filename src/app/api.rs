use rocket::time::OffsetDateTime;
use twitter_v2::authorization::BearerToken;
use twitter_v2::data::ReferencedTweetKind::RepliedTo;
use twitter_v2::id::NumericId;
use twitter_v2::query::{TweetField, UserField};
use twitter_v2::{Tweet, TwitterApi, User};

pub async fn get_tweets_from_user(user: &User) -> Vec<Tweet> {
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

pub async fn get_latest_tweet_from_user(user: &User) -> Tweet {
    load_api()
        .await
        .get_user_tweets(user.id)
        .max_results(5) //this line gets the max results
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
        .first()
        .expect("Couldn't parse latest tweet")
        .clone()
}

pub async fn get_new_tweets_from_user(user: &User, from: &OffsetDateTime) -> Vec<Tweet> {
    load_api()
        .await
        .get_user_tweets(user.id)
        .start_time(from.clone())
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

pub async fn get_tweet_by_id(id: u64) -> Option<Tweet> {
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
