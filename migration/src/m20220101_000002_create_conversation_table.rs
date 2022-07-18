use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20220101_000002_create_conversation_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Baker table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Conversation::Table)
                    .col(
                        ColumnDef::new(Conversation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Baker table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Conversation::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Conversation {
    Table,
    Id,
}