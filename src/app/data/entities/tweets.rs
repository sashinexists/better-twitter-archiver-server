//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use chrono::{DateTime, FixedOffset};
use rocket::time::OffsetDateTime;
use sea_orm::entity::prelude::*;
use twitter_v2::data::ReferencedTweet;

use crate::utils::TweetReferenceData;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tweets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub content: String,
    pub author_id: i64,
    pub conversation_id: i64,
    pub created_at: DateTime<FixedOffset>,
}

impl Model {
    pub fn to_tweet(&self, references:Vec<TweetReferenceData>) -> twitter_v2::Tweet {
        twitter_v2::Tweet {
            id: twitter_v2::id::NumericId::new(self.id.try_into().unwrap()),
            text: self.content.clone(),
            author_id: Some(twitter_v2::id::NumericId::new(
                self.author_id.try_into().unwrap(),
            )),
            conversation_id: Some(twitter_v2::id::NumericId::new(
                self.conversation_id.try_into().unwrap(),
            )),
            created_at: Some(
                OffsetDateTime::from_unix_timestamp(self.created_at.timestamp())
                    .expect("Failed time conversion"),
            ),
            attachments: None,
            context_annotations: None,
            entities: None,
            geo: None,
            in_reply_to_user_id: None,
            lang: None,
            non_public_metrics: None,
            organic_metrics: None,
            possibly_sensitive: None,
            promoted_metrics: None,
            public_metrics: None,
            referenced_tweets: Some(references.into_iter().map(|reference|reference.to_referenced_tweet()).collect::<Vec<ReferencedTweet>>()),
            reply_settings: None,
            source: None,
            withheld: None,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::conversations::Entity",
        from = "Column::ConversationId",
        to = "super::conversations::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Conversations,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

impl Related<super::conversations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversations.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
