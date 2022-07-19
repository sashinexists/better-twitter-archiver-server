use rocket::*;
mod app;
use app::data::entities::prelude::*;
use app::data::setup;
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
    app::data::read::tweets(db).await
}

#[get("/users")]
async fn users(db: &State<DatabaseConnection>) -> String {
    app::data::read::users(db).await
}

#[get("/user/<id>")]
async fn user_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    app::data::read::user_by_id(db, id).await
}

#[get("/tweet/<id>")]
async fn tweet_by_id(db: &State<DatabaseConnection>, id: i64) -> String {
    app::data::read::tweet_by_id(db, id).await
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
