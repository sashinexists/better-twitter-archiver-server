use sea_orm_migration::prelude::*;

use super::m20220101_000003_create_tweet_table::Tweet;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20220101_000004_create_tweet_reference_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Baker table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TweetReference::Table)
                    .col(
                        ColumnDef::new(TweetReference::SourceTweetId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TweetReference::ReferenceType).string().not_null())
                    .col(ColumnDef::new(TweetReference::ReferenceTweetId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tweet-reference-source_tweet_id")
                            .from(TweetReference::Table, TweetReference::SourceTweetId)
                            .to(Tweet::Table, Tweet::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tweet-reference-reference_tweet_id")
                            .from(TweetReference::Table, TweetReference::ReferenceTweetId)
                            .to(Tweet::Table, Tweet::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Baker table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TweetReference::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum TweetReference {
    Table,
    SourceTweetId,
    ReferenceType,
    ReferenceTweetId
}