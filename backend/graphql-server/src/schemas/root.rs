use juniper::{EmptySubscription, FieldError, FieldResult, RootNode};
use sea_orm::{
    entity::*, // , query::*
    // , ActiveValue::NotSet
    DatabaseConnection,
    InsertResult,
};

use crate::schemas;

pub struct Context {
    pub connection: DatabaseConnection,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    async fn get_latest_buzz(_context: &Context) -> FieldResult<i32> {
        todo!()
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    #[graphql(description = "Create User")]
    async fn create_user(
        user: schemas::users::UserModify,
        authentication_details: schemas::auth::AuthModify,
        context: &Context,
    ) -> FieldResult<schemas::users::UserDetails> {
        let connection = &context.connection;
        let auth_table = entity::auth::ActiveModel {
            contact_number: Set(authentication_details.contact_number),
            email: Set(authentication_details.email),
            password_version: Set(0.1),
            user_password: Set(authentication_details.password),
            username: Set(authentication_details.username),
            ..Default::default()
        };

        let auth_id_from_insert: InsertResult<entity::auth::ActiveModel> =
            entity::auth::Entity::insert(auth_table)
                .exec(connection)
                .await
                .unwrap();

        let user_table = entity::users::ActiveModel {
            auth_id: Set(auth_id_from_insert.last_insert_id),
            full_name: Set(user.full_name),
            profile_picture: Set(user.profile_picture),
            description: Set(user.description),
            location_or_region: Set(user.location_or_region),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let user_table = user_table.insert(connection).await.unwrap();

        Ok(crate::schemas::users::UserDetails {
            id: user_table.id as i32,
            auth_id: Some(user_table.auth_id as i32),
            full_name: user_table.full_name,
            description: user_table.description,
            profile_picture: user_table.profile_picture,
            location_or_region: user_table.location_or_region,
        })
    }
}
