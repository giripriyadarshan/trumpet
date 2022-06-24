use juniper::{EmptySubscription, FieldError, FieldResult, RootNode};
use sea_orm::{
    entity::*, // , query::*
    // , ActiveValue::NotSet
    DatabaseConnection,
    InsertResult,
};

use argonautica::Hasher;

use crate::schemas;

#[derive(Debug, Clone)]
pub struct Context {
    pub connection: DatabaseConnection,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    async fn get_latest_buzz(_context: &Context) -> FieldResult<i32> {
        Ok(1)
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

        let key = std::env::var("PASSWORD_SECRET_KEY").expect("SECRET_KEY must be set");

        let password = Hasher::default()
            .with_password(authentication_details.password)
            .with_secret_key(key)
            .hash()
            .unwrap();
        let auth_table = entity::auth::ActiveModel {
            contact_number: Set(authentication_details.contact_number),
            email: Set(authentication_details.email),
            password_version: Set(0.1),
            user_password: Set(password),
            username: Set(authentication_details.username),
            ..Default::default()
        };

        let auth_insert: Result<InsertResult<entity::auth::ActiveModel>, migration::DbErr> =
            entity::auth::Entity::insert(auth_table)
                .exec(connection)
                .await;

        match auth_insert {
            Ok(auth_id) => {
                let user_table = entity::users::ActiveModel {
                    auth_id: Set(auth_id.last_insert_id),
                    full_name: Set(user.full_name),
                    profile_picture: Set(user.profile_picture),
                    description: Set(user.description),
                    location_or_region: Set(user.location_or_region),
                    created_at: Set(chrono::Utc::now()),
                    ..Default::default()
                };

                let user_table = user_table.insert(connection).await;

                match user_table {
                    Ok(user_table) => Ok(schemas::users::UserDetails {
                        id: user_table.id as i32,
                        auth_id: Some(user_table.auth_id as i32),
                        full_name: user_table.full_name,
                        description: user_table.description,
                        profile_picture: user_table.profile_picture,
                        location_or_region: user_table.location_or_region,
                    }),

                    Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                }
            }
            Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
        }
    }

    #[graphql(description = "delete user")]
    async fn delete_user(_jwt: String, _context: &Context) -> FieldResult<bool> {
        Ok(true)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
