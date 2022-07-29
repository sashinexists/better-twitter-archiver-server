use chrono::FixedOffset;
use rocket::*;
mod app;

use app::data::setup;
use dotenvy::dotenv;

use sea_orm::{DatabaseConnection, DbErr};
mod utils;

#[get("/")]
async fn index() -> &'static str {
    "Welcome to the better twitter archiver server!"
}

#[get("/tweets")]
async fn tweets(db: &State<DatabaseConnection>) -> String {
    utils::to_ron(&app::data::read::tweets(db).await)
}

#[get("/users")]
async fn users(db: &State<DatabaseConnection>) -> String {
    utils::to_ron(&app::data::read::users(db).await)
}

#[get("/userbyid/<id>")]
async fn user_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    utils::to_ron(&app::load_user_from_id(db, id).await)
}

#[get("/user/<twitter_handle>")]
async fn user_by_twitter_handle(db: &State<DatabaseConnection>, twitter_handle: &str) -> String {
    utils::to_ron(&app::load_user_from_twitter_handle(db, &twitter_handle).await)
}

#[get("/user/<twitter_handle>/info")]
async fn user_info_by_twitter_handle(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
) -> String {
    let output = utils::to_ron(&app::load_user_from_twitter_handle(db, &twitter_handle).await);
    println!("{}", output);
    output
}
//you may wish to get rid of this route
#[get("/user/<twitter_handle>/latest")]
async fn users_latest_tweet_by_id(db: &State<DatabaseConnection>, twitter_handle: &str) -> String {
    utils::to_ron(&app::load_offset_datetime_for_users_latest_tweet(db, &twitter_handle).await)
}

#[get("/user/<twitter_handle>/has_tweeted_since/<rfc3339_date>")]
async fn has_user_tweeted_since_date(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
    rfc3339_date: &str,
) -> String {
    let date_timestamp = chrono::DateTime::<FixedOffset>::parse_from_rfc3339(rfc3339_date)
        .expect("Failed to parse date")
        .timestamp();
    utils::to_ron(&app::has_user_tweeted_since_date(db, &twitter_handle, date_timestamp).await)
}

#[get("/user/<twitter_handle>/tweets-since/<rfc3339_date>")]
async fn users_tweets_since_date(
    db: &State<DatabaseConnection>,
    twitter_handle: &str,
    rfc3339_date: &str,
) -> String {
    utils::to_ron(&app::load_users_tweets_since_date(db, &twitter_handle, rfc3339_date).await)
}

#[get("/user/<twitter_handle>/tweets")]
async fn users_tweets(db: &State<DatabaseConnection>, twitter_handle: &str) -> String {
    utils::to_ron(&app::load_user_tweets_from_twitter_handle(db, twitter_handle).await)
}

#[get("/tweet/<id>")]
async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    utils::to_ron(&app::load_tweet_from_id(db, id).await)
}

#[get("/conversation/<id>")]
async fn conversation_by_tweet_id(db: &State<DatabaseConnection>, id: i64) -> String {
    utils::to_ron(&app::load_twitter_conversation_from_tweet_id(db, id).await)
}

#[get("/search/<query>")]
async fn search_tweets_in_db(db: &State<DatabaseConnection>, query: &str) -> String {
    utils::to_ron(&app::search_tweets_in_db(db, query).await)
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let db = match setup::set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    rocket::build().manage(db).mount(
        "/",
        // Don't forget to mount the new endpoint handlers
        routes![
            index,
            tweets,
            tweet_by_id,
            users,
            user_by_id,
            user_by_twitter_handle,
            users_tweets,
            user_info_by_twitter_handle,
            conversation_by_tweet_id,
            users_tweets_since_date,
            has_user_tweeted_since_date,
            search_tweets_in_db
        ],
    )
}

#[derive(Responder)]
#[response(status = 500, content_type = "json")]
struct ErrorResponder {
    message: String,
}

// The following impl's are for easy conversion of error types.

#[allow(clippy::from_over_into)]
impl Into<ErrorResponder> for DbErr {
    fn into(self) -> ErrorResponder {
        ErrorResponder {
            message: self.to_string(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ErrorResponder> for String {
    fn into(self) -> ErrorResponder {
        ErrorResponder { message: self }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ErrorResponder> for &str {
    fn into(self) -> ErrorResponder {
        self.to_owned().into()
    }
}
