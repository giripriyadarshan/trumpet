use sea_orm_migration::prelude::*;

use entity::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(auth::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(auth::Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(auth::Column::Username)
                            .string_len(255)
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(auth::Column::Email)
                            .string_len(255)
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(auth::Column::ContactNumber)
                            .string_len(255)
                            .unique_key(),
                    )
                    .col(ColumnDef::new(auth::Column::UserPassword).text().not_null())
                    .col(
                        ColumnDef::new(auth::Column::PasswordVersion)
                            .double()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(users::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(users::Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(users::Column::AuthId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(users::Entity, users::Column::AuthId)
                            .to(auth::Entity, auth::Column::Id),
                    )
                    .col(ColumnDef::new(users::Column::FullName).text().not_null())
                    .col(ColumnDef::new(users::Column::ProfilePicture).text())
                    .col(ColumnDef::new(users::Column::Description).text())
                    .col(
                        ColumnDef::new(users::Column::LocationOrRegion)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(users::Column::Following).array("BIGINT".to_string()))
                    .col(ColumnDef::new(users::Column::Followers).array("BIGINT".to_string()))
                    .col(
                        ColumnDef::new(users::Column::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(ratings::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ratings::Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ratings::Column::IsBuzz).boolean().not_null())
                    .col(
                        ColumnDef::new(ratings::Column::ModuleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ratings::Column::Upvotes)
                            .big_integer()
                            .default(0 as i64),
                    )
                    .col(
                        ColumnDef::new(ratings::Column::Views)
                            .big_integer()
                            .default(0 as i64),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(buzz::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(buzz::Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(buzz::Column::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(buzz::Entity, buzz::Column::UserId)
                            .to(users::Entity, users::Column::Id),
                    )
                    .col(ColumnDef::new(buzz::Column::Description).text().not_null())
                    .col(ColumnDef::new(buzz::Column::ImageLink).text())
                    .col(ColumnDef::new(buzz::Column::VideoLink).text())
                    .col(ColumnDef::new(buzz::Column::BuzzWords).array("TEXT".to_string()))
                    .col(ColumnDef::new(buzz::Column::MentionedUsers).array("BIGINT".to_string()))
                    .col(ColumnDef::new(buzz::Column::RatingsId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(buzz::Entity, buzz::Column::RatingsId)
                            .to(ratings::Entity, ratings::Column::Id),
                    )
                    .col(
                        ColumnDef::new(buzz::Column::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(reply::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(reply::Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(reply::Column::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(reply::Entity, reply::Column::UserId)
                            .to(users::Entity, users::Column::Id),
                    )
                    .col(
                        ColumnDef::new(reply::Column::BuzzId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(reply::Entity, reply::Column::BuzzId)
                            .to(buzz::Entity, buzz::Column::Id),
                    )
                    .col(
                        ColumnDef::new(reply::Column::ReplyContent)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(reply::Column::BuzzWords).array("TEXT".to_string()))
                    .col(ColumnDef::new(reply::Column::MentionedUsers).array("BIGINT".to_string()))
                    .col(ColumnDef::new(reply::Column::RatingsId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(reply::Entity, reply::Column::RatingsId)
                            .to(ratings::Entity, ratings::Column::Id),
                    )
                    .col(
                        ColumnDef::new(reply::Column::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(trending::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(trending::Column::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(trending::Column::TrendingId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(trending::Column::Description).text())
                    .col(ColumnDef::new(trending::Column::BuzzWords).array("TEXT".to_string()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(trending::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(reply::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(buzz::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(ratings::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(users::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(auth::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}
