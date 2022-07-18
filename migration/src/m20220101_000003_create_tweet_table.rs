// m20220101_000001_create_Tweets_table.rs

use sea_orm_migration::prelude::*;
use super::m20220101_000001_create_user_table::User;
use super::m20220101_000002_create_conversation_table::Conversation;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000003_create_tweet_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Tweets table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tweet::Table)
                    .col(
                        ColumnDef::new(Tweet::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tweet::ConversationId).string().not_null())
                    .col(ColumnDef::new(Tweet::Content).string().not_null())
                    .col(ColumnDef::new(Tweet::AuthorId).integer().not_null())
                    .col(ColumnDef::new(Tweet::CreatedAt).date().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tweet-author_id")
                            .from(Tweet::Table, Tweet::AuthorId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tweet-conversation_id")
                            .from(Tweet::Table, Tweet::ConversationId)
                            .to(Conversation::Table, Conversation::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Tweets table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tweet::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Tweet {
    Table,
    Id,
    AuthorId,
    ConversationId,
    CreatedAt,
    Content
}