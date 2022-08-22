use rocket::time::OffsetDateTime;
use twitter_v2::authorization::BearerToken;
use twitter_v2::query::{TweetField, UserField};
use twitter_v2::{Tweet, TwitterApi, User};

pub async fn get_tweets_from_user(user: &User) -> Vec<Tweet> {
    let twitter_handle = &user.username;
    load_api()
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
        })
}

pub async fn get_latest_tweet_from_user(user: &User) -> Tweet {
    let twitter_handle = &user.username;
    load_api()
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
        .clone()
}

pub async fn get_new_tweets_from_user(user: &User, from: &OffsetDateTime) -> Vec<Tweet> {
    let twitter_handle = &user.username;
    load_api()
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
        })
}

pub async fn get_all_tweets_from_user(user: &User) -> Vec<Tweet> {
    let twitter_handle = &user.username;
    let mut output = get_first_hundred_tweets_from_user(user).await;
    const FIRST_TWEET_ID: u64 = 1012187366587392000; // 1490542591154130947; //@yudapearls first tweet id = 1012187366587392000
    let mut last_id = output
        .last()
        .unwrap_or_else(|| {
            panic!("Failed to get the last tweet from the @{twitter_handle}'s first hundred.",)
        })
        .id
        .as_u64();
    let mut i = 1;
    while i < 32 {
        output.append(&mut get_tweets_from_user_until_id(user, last_id).await);
        last_id = output
            .last()
            .unwrap_or_else(|| {
                panic!(
                    "Failed to get the last tweet from this batch of @{twitter_handle}'s tweets.",
                )
            })
            .id
            .as_u64();
        println!("Loading tweets up to {i}00");
        i += 1;
    }

    output
}

pub async fn get_first_hundred_tweets_from_user(user: &User) -> Vec<Tweet> {
    let twitter_handle = &user.username;
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
        .unwrap_or_else(|error| panic!("Failed to load @{twitter_handle}'s first hundred tweets from the twitter api. \n\nError: {:?}", error))
        .into_data()
        .unwrap_or_else(|| {
            panic!("Failed to open @{twitter_handle}'s first hunrdred tweets after fetching them from the twitter api.")
        })
}

pub async fn get_tweets_from_user_until_id(user: &User, id: u64) -> Vec<Tweet> {
    let twitter_handle = &user.username;
    load_api()
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
        })
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
        .unwrap_or_else(|error| panic!("Failed to get tweet of id {id} from the twitter api. \n\nError: {:?}", error))
        .into_data()
}


pub async fn get_user_by_twitter_handle(twitter_handle: &str) -> User {
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
        })
}

pub async fn get_user_by_id(id: u64) -> User {
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
        })
}

pub async fn load_api() -> TwitterApi<BearerToken> {
    let auth = BearerToken::new(std::env::var("TWITTER_DEV_BEARER_TOKEN").unwrap());
    TwitterApi::new(auth)
}
