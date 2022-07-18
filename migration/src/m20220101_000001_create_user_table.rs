use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20220101_000001_create_user_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Baker table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Username).string().not_null())
                    .col(ColumnDef::new(User::Name).string().not_null())
                    .col(ColumnDef::new(User::Description).string().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Baker table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Username,
    Name,
    Description,
}