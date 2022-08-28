use chrono::{DateTime, FixedOffset};
use futures::{future::join_all, StreamExt};
use rocket::{
    serde::Serialize,
    time::{format_description, OffsetDateTime},
    State,
};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use twitter_v2::{
    data::{ReferencedTweet, ReferencedTweetKind},
    id::NumericId,
    Tweet, User,
};

use crate::app::data::entities::prelude::*;
use crate::app::data::entities::*;

pub struct TweetData {
    pub tweet: Option<tweets::Model>,
    pub references: Vec<tweet_references::Model>,
}

impl TweetData {
    pub fn new(tweet: tweets::Model, references: Vec<tweet_references::Model>) -> Self {
        TweetData {
            tweet: Some(tweet),
            references,
        }
    }

    pub fn empty() -> Self {
        Self {
            tweet: None,
            references: Vec::new(),
        }
    }

    pub async fn read(db: &State<DatabaseConnection>, id: i64) -> Self {
        let db = db as &DatabaseConnection;
        let references = TweetReferences::find()
            .filter(tweet_references::Column::SourceTweetId.eq(id))
            .all(db)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to get tweet references for tweet of id {id}. Error: {:?}",
                    error
                )
            })
            .into_iter()
            .collect();
        let tweet = Tweets::find_by_id(id)
            .one(db)
            .await
            .unwrap_or_else(|error| {
                panic!("Failed to get tweet {id} from database. Error: {:?}", error)
            });

        Self { tweet, references }
    }

    pub async fn read_from_data_model(
        db: &State<DatabaseConnection>,
        tweet_model: tweets::Model,
    ) -> Self {
        let db = db as &DatabaseConnection;
        let references = TweetReferences::find()
            .filter(tweet_references::Column::SourceTweetId.eq(tweet_model.id))
            .all(db)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to get tweet references for tweet of id {}. Error: {:?}",
                    tweet_model.id, error
                )
            })
            .into_iter()
            .collect();
        Self {
            tweet: Some(tweet_model),
            references,
        }
    }

    pub async fn from_api_tweet(tweet: Option<Tweet>) -> Self {
        if let Some(tweet) = tweet {
            let references_data: Vec<TweetReferenceData> = tweet
                .referenced_tweets
                .unwrap_or_else(|| panic!("Failed to get references for tweet of id {}", tweet.id))
                .iter()
                .map(|reference| {
                    TweetReferenceData::from_referenced_tweet(
                        u64_to_i64(tweet.id.as_u64()),
                        reference,
                    )
                })
                .collect();
            let references: Vec<tweet_references::Model> = references_data
                .into_iter()
                .map(|reference| tweet_references::Model {
                    source_tweet_id: reference.source_tweet_id,
                    reference_type: TweetReferenceData::type_to_string(&reference),
                    referenced_tweet_id: reference.reference_tweet_id,
                })
                .collect();
            Self {
                tweet: Some(tweets::Model {
                    id: u64_to_i64(tweet.id.as_u64()),
                    content: tweet.text,
                    author_id: u64_to_i64(
                        tweet
                            .author_id
                            .unwrap_or_else(|| {
                                panic!("Failed to get author_id for tweet of id {}.\n", tweet.id)
                            })
                            .as_u64(),
                    ),
                    conversation_id: u64_to_i64(
                        tweet
                            .conversation_id
                            .unwrap_or_else(|| {
                                panic!(
                                    "Failed to get conversation_id for tweet of id {}.\n",
                                    tweet.id
                                )
                            })
                            .as_u64(),
                    ),
                    created_at: convert_date_to_chrono(tweet.created_at),
                }),
                references,
            }
        } else {
            TweetData {
                tweet: None,
                references: Vec::new(),
            }
        }
    }

    pub async fn read_many(db: &State<DatabaseConnection>, ids: &[i64]) -> Vec<Self> {
        join_all(ids.into_iter().map(|id| Self::read(db, *id))).await
    }

    pub async fn write(&self, db: &State<DatabaseConnection>) {
        if let Some(tweet) = self.tweet.clone() {
            let to_write = tweets::ActiveModel {
                id: ActiveValue::set(tweet.id),
                conversation_id: ActiveValue::set(tweet.conversation_id),
                content: ActiveValue::set(tweet.content),
                author_id: ActiveValue::set(tweet.author_id),
                created_at: ActiveValue::set(tweet.created_at),
            };

            let res = Tweets::insert(to_write).exec(db.inner()).await;

            match res {
                Ok(_res) => (),
                Err(_error) => println!(
                    "Failed to to write tweet of id {} to the database.",
                    tweet.id
                ),
            }
            let tweet_reference_stream = futures::stream::iter(self.references.iter());
            tweet_reference_stream
                .for_each(|tweet_ref| async {
                    let to_write = tweet_references::ActiveModel {
                        source_tweet_id: ActiveValue::set(tweet_ref.source_tweet_id),
                        reference_type: ActiveValue::set(tweet_ref.reference_type.clone()),
                        referenced_tweet_id: ActiveValue::set(tweet_ref.referenced_tweet_id),
                    };
                    let res = TweetReferences::insert(to_write).exec(db.inner()).await;

                    match res {
                        Ok(_res) => (),
                        Err(_e) => println!(
                            "Failed to add tweet references for tweet {} to the database.",
                            tweet.id
                        ),
                    }
                })
                .await;
        }
    }

    pub async fn write_many(db: &State<DatabaseConnection>, tweets: Vec<&Self>) {
        let tweets = futures::stream::iter(tweets.iter());
        tweets
            .for_each(|tweet| async {
                tweet.write(db).await;
            })
            .await;
    }
}

pub struct UserData {
    pub user: Option<users::Model>,
}

impl UserData {
    pub async fn from_api_user(api_user: &User) -> Self {
        Self {
            user: Some(users::Model {
                id: u64_to_i64(api_user.id.as_u64()),
                name: api_user.name.clone(),
                username: api_user.username.clone(),
                description: api_user.description.clone().unwrap_or_else(|| {
                    panic!("Failed to parse description for @{}", api_user.username)
                }),
            }),
        }
    }

    pub async fn empty() ->Self {
        Self {
            user: None
        }
    }

    pub async fn from_data_model(user_from_db: users::Model) -> Self {
        UserData {
            user: Some(user_from_db),
        }
    }

    pub async fn read(db: &State<DatabaseConnection>, id: i64) -> Self {
        let db = db as &DatabaseConnection;
        let user = Users::find_by_id(id).one(db).await.unwrap_or_else(|error| {
            panic!("Failed to get tweet {id} from database. Error: {:?}", error)
        });

        Self { user }
    }

    pub async fn read_from_twitter_handle(
        db: &State<DatabaseConnection>,
        twitter_handle: &str,
    ) -> Self {
        let db = db as &DatabaseConnection;
        let user = Users::find()
            .filter(users::Column::Username.eq(twitter_handle))
            .one(db)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to read user @{twitter_handle} from database. Error: {:?}",
                    error
                )
            });
        Self { user }
    }

    pub async fn write(&self, db: &State<DatabaseConnection>) {
        if let Some(user) = self.user.clone() {
            let to_write = users::ActiveModel {
                id: ActiveValue::set(user.id),
                name: ActiveValue::set(user.name),
                username: ActiveValue::set(user.username.clone()),
                description: ActiveValue::set(user.description),
            };

            let res = Users::insert(to_write).exec(db.inner()).await;

            match res {
                Ok(_res) => (),
                Err(_error) => println!(
                    "Failed to to write user @{} to the database.",
                    user.username
                ),
            }
        }
    }
}

pub struct ConversationData {
    id: i64,
    tweets: Vec<TweetData>,
}

pub fn convert_date_to_chrono(date: Option<OffsetDateTime>) -> DateTime<FixedOffset> {
    let format = format_description::parse(
        "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour \
             sign:mandatory]:[offset_minute]",
    )
    .expect("Bad formatter");

    let date_string = date
        .expect("Couldn't get the tweets date")
        .format(&format)
        .expect("Couldn't parse with thes formatter");

    chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&date_string)
        .expect("failed to parse date from string")
}

pub fn to_ron<T: ?Sized + Serialize>(item: &T) -> String {
    ron::ser::to_string_pretty(item, ron::ser::PrettyConfig::new())
        .expect("Failed to parse tweet into string")
}
#[derive(Debug, Serialize)]
pub struct TweetReferenceData {
    pub reference_type: ReferencedTweetKind,
    pub source_tweet_id: i64,
    pub reference_tweet_id: i64,
}

impl TweetReferenceData {
    pub fn type_to_string(&self) -> String {
        match self.reference_type {
            ReferencedTweetKind::RepliedTo => "replied_to",
            ReferencedTweetKind::Retweeted => "retweeted",
            ReferencedTweetKind::Quoted => "quoted",
        }
        .to_string()
    }

    pub fn kind_from_string(input: &str) -> Option<ReferencedTweetKind> {
        match input {
            "replied_to" => Some(ReferencedTweetKind::RepliedTo),
            "retweeted" => Some(ReferencedTweetKind::Retweeted),
            "quoted" => Some(ReferencedTweetKind::Quoted),
            _ => None,
        }
    }

    pub fn from_referenced_tweet(id: i64, referenced_tweet: &ReferencedTweet) -> Self {
        Self {
            reference_type: referenced_tweet.kind.clone(),
            source_tweet_id: id.clone(),
            reference_tweet_id: referenced_tweet
                .id
                .as_u64()
                .try_into()
                .expect("Bad referenced tweet id"),
        }
    }

    pub fn to_referenced_tweet(&self) -> ReferencedTweet {
        ReferencedTweet {
            kind: self.reference_type,
            id: NumericId::from(i64_to_u64(self.reference_tweet_id)),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            reference_type: self.reference_type.clone(),
            source_tweet_id: self.source_tweet_id.clone(),
            reference_tweet_id: self.reference_tweet_id.clone(),
        }
    }
}

pub fn i64_to_u64(i: i64) -> u64 {
    i.try_into()
        .unwrap_or_else(|error| panic!("Failed to parse u64 from i64. Error:\n{error}"))
}

pub fn u64_to_i64(u: u64) -> i64 {
    u.try_into()
        .unwrap_or_else(|error| panic!("Failed to parse i64 from u64. Error:\n{error}"))
}
