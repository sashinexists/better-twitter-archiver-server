use rocket::*;
mod entities;
mod setup;

use entities::prelude::*;
use futures::executor::block_on;
use rocket::serde::json::Json;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, DbErr, DeleteResult, EntityTrait, InsertResult, QueryFilter, Statement,
};
#[get("/")]
async fn index() -> &'static str {
    "Hello there!"
}

#[get("/tweets")]
async fn tweets(db: &State<DatabaseConnection>) -> String {
    let db = db as &DatabaseConnection;

    let tweets: Vec<twitter_v2::Tweet> = Tweets::find()
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|b| b.to_tweet())
        .collect::<Vec<twitter_v2::Tweet>>();

    ron::ser::to_string_pretty(&tweets, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

#[get("/users")]
async fn users(db: &State<DatabaseConnection>) -> String {
    let db = db as &DatabaseConnection;

    let users: Vec<twitter_v2::User> = Users::find()
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|b| b.to_twitter_user())
        .collect::<Vec<twitter_v2::User>>();

    ron::ser::to_string_pretty(&users, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

#[get("/user/<id>")]
async fn user_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    let db = db as &DatabaseConnection;

    let user = Users::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet")
        .expect("Failed to open the option model tweet")
        .to_twitter_user();

    ron::ser::to_string_pretty(&user, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

#[get("/tweet/<id>")]
async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    let db = db as &DatabaseConnection;

    let tweet = Tweets::find_by_id(id)
        .one(db)
        .await
        .expect("Failed to open the result option model tweet")
        .expect("Failed to open the option model tweet")
        .to_tweet();

    ron::ser::to_string_pretty(&tweet, ron::ser::PrettyConfig::new()).expect("Failed to parse ron")
}

#[launch]
async fn rocket() -> _ {
    let db = match setup::set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    rocket::build().manage(db).mount(
        "/",
        // Don't forget to mount the new endpoint handlers
        routes![index, tweets, tweet_by_id, users, user_by_id],
    )
}
