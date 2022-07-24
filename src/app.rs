use rocket::State;
use sea_orm::DatabaseConnection;

use self::data::entities::tweets;

pub mod api;
pub mod data;

pub async fn load_tweet_from_id(db: &State<DatabaseConnection>, id: i64) -> String {
    match data::read::tweet_by_id(db, id).await {
        Some(tweet_string) => tweet_string,
        None => {
            let tweet = api::get_tweet_by_id(id.try_into().unwrap()).await;
            data::write::tweet(db, &tweet).await;
            ron::ser::to_string_pretty(&tweet, ron::ser::PrettyConfig::new())
                .expect("Failed to parse tweet into string")
        }
    }
}
