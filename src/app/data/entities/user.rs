//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub username: String,
    pub name: String,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tweet::Entity")]
    Tweet,
}

impl Related<super::tweet::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tweet.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}