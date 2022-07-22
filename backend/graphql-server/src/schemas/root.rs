use juniper::{EmptySubscription, FieldError, FieldResult, RootNode};
use sea_orm::{entity::*, query::*, DatabaseConnection, DbErr, InsertResult};

use crate::lib::{
    common::*,
    server_auth::{
        authenticate,
        AuthenticationStatus::{Authenticated, Unauthenticated},
    },
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
    async fn get_buzzes(
        page_details: schemas::buzz::GetAllBuzzInput,
        context: &Context,
    ) -> FieldResult<schemas::buzz::AllBuzzResult> {
        let connection = &context.connection;

        let paginated_posts = match page_details.user_id {
            Some(user_id) => {
                let posts = entity::buzz::Entity::find()
                    .filter(entity::buzz::Column::UserId.eq(user_id.parse::<i64>().unwrap()))
                    .order_by(entity::buzz::Column::CreatedAt, Order::Desc)
                    .paginate(connection, page_details.page_size as usize);
                posts
            }

            None => {
                let posts = entity::buzz::Entity::find()
                    .order_by(entity::buzz::Column::CreatedAt, Order::Desc)
                    .paginate(connection, page_details.page_size as usize);
                posts
            }
        };

        let total_pages = paginated_posts.num_pages().await.unwrap() as i32;
        let total_buzzes = paginated_posts.num_items().await.unwrap() as i32;

        let fetch_buzzes: Result<Vec<entity::buzz::Model>, DbErr> = paginated_posts
            .fetch_page((page_details.page_number - 1) as usize)
            .await;

        return match fetch_buzzes {
            Ok(buzzes) => {
                let return_buzzes: Vec<schemas::buzz::BuzzResult> = buzzes
                    .into_iter()
                    .map(|buzz| schemas::buzz::BuzzResult {
                        id: buzz.id.to_string(),
                        user_id: buzz.user_id.to_string(),
                        description: buzz.description.to_string(),
                        image_link: buzz.image_link,
                        video_link: buzz.video_link,
                        buzz_words: buzz.buzz_words,
                        mentioned_users: buzz.mentioned_users,
                        ratings_id: Some(buzz.ratings_id.unwrap_or(-1).to_string()),
                        created_at: buzz.created_at,
                    })
                    .collect();

                Ok(schemas::buzz::AllBuzzResult {
                    buzzes: return_buzzes,
                    total_buzzes,
                    total_pages,
                    page_number: page_details.page_number,
                    page_size: page_details.page_size,
                })
            }

            Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
        };
    }

    async fn get_buzz_details(
        id: String,
        context: &Context,
    ) -> FieldResult<schemas::buzz::BuzzResult> {
        let connection = &context.connection;

        let buzz = entity::buzz::Entity::find()
            .filter(entity::buzz::Column::Id.eq(id.parse::<i64>().unwrap()))
            .one(connection)
            .await;

        return match buzz {
            Ok(buzz) => match buzz {
                Some(buzz) => Ok(schemas::buzz::BuzzResult {
                    id: buzz.id.to_string(),
                    user_id: buzz.user_id.to_string(),
                    description: buzz.description.to_string(),
                    image_link: buzz.image_link,
                    video_link: buzz.video_link,
                    buzz_words: buzz.buzz_words,
                    mentioned_users: buzz.mentioned_users,
                    ratings_id: Some(buzz.ratings_id.unwrap_or(-1).to_string()),
                    created_at: buzz.created_at,
                }),

                None => Err(FieldError::new("Buzz not found", juniper::Value::Null)),
            },

            Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
        };
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

        let auth_insert: Result<InsertResult<entity::auth::ActiveModel>, DbErr> =
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
                    created_at: Set(chrono::DateTime::from(chrono::Utc::now())),
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
    async fn delete_user(jwt: String, context: &Context) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;
        match authentication {
            Authenticated(authentication) => {
                if authentication.is_one_time_jwt {
                    let user = entity::users::Entity::find_by_id(authentication.user_id)
                        .one(connection)
                        .await;
                    match user {
                        Ok(user) => {
                            let user = user.unwrap();
                            let user = user.delete(connection).await;
                            return match user {
                                Ok(_) => {
                                    let auth =
                                        entity::auth::Entity::find_by_id(authentication.auth_id)
                                            .one(connection)
                                            .await;
                                    match auth {
                                        Ok(auth) => {
                                            let auth = auth.unwrap();
                                            let auth = auth.delete(connection).await;
                                            match auth {
                                                Ok(_) => Ok(true),
                                                Err(e) => Err(FieldError::new(
                                                    e.to_string(),
                                                    juniper::Value::Null,
                                                )),
                                            }
                                        }
                                        Err(e) => Err(FieldError::new(
                                            e.to_string(),
                                            juniper::Value::Null,
                                        )),
                                    }
                                }
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            };
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Authentication Failed",
                        juniper::Value::Null,
                    ))
                }
            }

            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        }
    }

    #[graphql(description = "update user")]
    async fn update_user(
        jwt: String,
        user_modify: schemas::users::UserModify,
        context: &Context,
    ) -> FieldResult<schemas::users::UserDetails> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                let user = entity::users::Entity::find_by_id(authenticated.user_id)
                    .one(connection)
                    .await;
                match user {
                    Ok(user) => {
                        let mut user: entity::users::ActiveModel = user.unwrap().into();

                        user.full_name = Set(user_modify.full_name);
                        user.description = Set(user_modify.description);
                        user.profile_picture = Set(user_modify.profile_picture);
                        user.location_or_region = Set(user_modify.location_or_region);

                        let user: Result<entity::users::Model, DbErr> =
                            user.update(connection).await;

                        match user {
                            Ok(user) => Ok(schemas::users::UserDetails {
                                id: user.id as i32,
                                auth_id: Some(user.auth_id as i32),
                                full_name: user.full_name,
                                description: user.description,
                                profile_picture: user.profile_picture,
                                location_or_region: user.location_or_region,
                            }),
                            Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                        }
                    }

                    Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "change username")]
    async fn change_username(
        jwt: String,
        username: String,
        context: &Context,
    ) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.is_one_time_jwt {
                    let auth = entity::auth::Entity::find_by_id(authenticated.auth_id)
                        .one(connection)
                        .await;
                    match auth {
                        Ok(auth) => {
                            let mut auth: entity::auth::ActiveModel = auth.unwrap().into();

                            auth.username = Set(username);
                            let auth: Result<entity::auth::Model, DbErr> =
                                auth.update(connection).await;
                            match auth {
                                Ok(_) => Ok(true),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Authentication Failed",
                        juniper::Value::Null,
                    ))
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "change password")]
    async fn change_password(
        jwt: String,
        password: String,
        context: &Context,
    ) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;
        let key = std::env::var("PASSWORD_SECRET_KEY").expect("SECRET_KEY must be set");

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.is_one_time_jwt {
                    let auth = entity::auth::Entity::find_by_id(authenticated.auth_id)
                        .one(connection)
                        .await;
                    match auth {
                        Ok(auth) => {
                            let mut auth: entity::auth::ActiveModel = auth.unwrap().into();

                            let password = Hasher::default()
                                .with_password(password)
                                .with_secret_key(key)
                                .hash()
                                .unwrap();

                            auth.user_password = Set(password);
                            let auth: Result<entity::auth::Model, DbErr> =
                                auth.update(connection).await;
                            match auth {
                                Ok(_) => Ok(true),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Authentication Failed",
                        juniper::Value::Null,
                    ))
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "change email")]
    async fn change_email(jwt: String, email: String, context: &Context) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.is_one_time_jwt {
                    let auth = entity::auth::Entity::find_by_id(authenticated.auth_id)
                        .one(connection)
                        .await;
                    match auth {
                        Ok(auth) => {
                            let mut auth: entity::auth::ActiveModel = auth.unwrap().into();

                            auth.email = Set(email);
                            let auth: Result<entity::auth::Model, DbErr> =
                                auth.update(connection).await;
                            match auth {
                                Ok(_) => Ok(true),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Authentication Failed",
                        juniper::Value::Null,
                    ))
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "change contact number")]
    async fn change_contact_number(
        jwt: String,
        contact_number: String,
        context: &Context,
    ) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.is_one_time_jwt {
                    let auth = entity::auth::Entity::find_by_id(authenticated.auth_id)
                        .one(connection)
                        .await;
                    match auth {
                        Ok(auth) => {
                            let mut auth: entity::auth::ActiveModel = auth.unwrap().into();

                            auth.contact_number = Set(Some(contact_number));
                            let auth: Result<entity::auth::Model, DbErr> =
                                auth.update(connection).await;
                            match auth {
                                Ok(_) => Ok(true),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Authentication Failed",
                        juniper::Value::Null,
                    ))
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "logout from all devices")]
    async fn logout_from_all_devices(jwt: String, context: &Context) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.is_one_time_jwt {
                    let auth = entity::auth::Entity::find_by_id(authenticated.auth_id)
                        .one(connection)
                        .await;
                    match auth {
                        Ok(auth) => {
                            let mut auth: entity::auth::ActiveModel = auth.unwrap().into();

                            auth.password_version = Set(auth.password_version.unwrap() + 0.1_f64);
                            let auth: Result<entity::auth::Model, DbErr> =
                                auth.update(connection).await;
                            match auth {
                                Ok(_) => Ok(true),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Authentication Failed",
                        juniper::Value::Null,
                    ))
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "create a buzz")]
    async fn create_buzz(
        jwt: String,
        buzz: schemas::buzz::BuzzInput,
        context: &Context,
    ) -> FieldResult<schemas::buzz::BuzzResult> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.user_id.to_string() == buzz.user_id {
                    let ratings_table = entity::ratings::ActiveModel {
                        ..Default::default()
                    };

                    let ratings_insert: Result<InsertResult<entity::ratings::ActiveModel>, DbErr> =
                        entity::ratings::Entity::insert(ratings_table)
                            .exec(connection)
                            .await;

                    match ratings_insert {
                        Ok(ratings) => {
                            let buzz_table = entity::buzz::ActiveModel {
                                user_id: Set(buzz.user_id.to_string().parse::<i64>().unwrap()),
                                description: Set(buzz.description),
                                image_link: Set(buzz.image_link),
                                video_link: Set(buzz.video_link),
                                buzz_words: Set(buzz.buzz_words),
                                mentioned_users: Set(buzz.mentioned_users),
                                ratings_id: Set(Some(ratings.last_insert_id)),
                                created_at: Set(chrono::DateTime::from(chrono::Utc::now())),

                                ..Default::default()
                            };

                            let buzz_insert = buzz_table.insert(connection).await;

                            match buzz_insert {
                                Ok(buzz) => Ok(schemas::buzz::BuzzResult {
                                    id: buzz.id.to_string(),
                                    user_id: buzz.user_id.to_string(),
                                    description: buzz.description,
                                    image_link: buzz.image_link,
                                    video_link: buzz.video_link,
                                    buzz_words: buzz.buzz_words,
                                    mentioned_users: buzz.mentioned_users,
                                    ratings_id: Some(buzz.ratings_id.unwrap_or(-1).to_string()),
                                    created_at: buzz.created_at,
                                }),

                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Cant create buzz on behalf of other users",
                        juniper::Value::Null,
                    ))
                }
            }

            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "delete buzz")]
    async fn delete_buzz(jwt: String, buzz_id: String, context: &Context) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                let get_buzz = entity::buzz::Entity::find_by_id(buzz_id.parse::<i64>().unwrap())
                    .one(connection)
                    .await;
                match get_buzz {
                    Ok(buzz) => match buzz {
                        Some(buzz) => {
                            if buzz.user_id.to_string() == authenticated.user_id.to_string() {
                                let buzz_delete = entity::buzz::Entity::delete_by_id(
                                    buzz_id.parse::<i64>().unwrap(),
                                )
                                .exec(connection)
                                .await;
                                match buzz_delete {
                                    Ok(_) => Ok(true),
                                    Err(e) => {
                                        Err(FieldError::new(e.to_string(), juniper::Value::Null))
                                    }
                                }
                            } else {
                                Err(FieldError::new(
                                    "Cant delete buzz on behalf of other users",
                                    juniper::Value::Null,
                                ))
                            }
                        }
                        None => Err(FieldError::new("Buzz not found", juniper::Value::Null)),
                    },
                    Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "create reply")]
    async fn create_reply(
        jwt: String,
        reply: schemas::reply::ReplyInput,
        context: &Context,
    ) -> FieldResult<schemas::reply::ReplyResult> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                if authenticated.user_id.to_string() == reply.user_id {
                    let ratings_table = entity::ratings::ActiveModel {
                        ..Default::default()
                    };

                    let ratings_insert: Result<InsertResult<entity::ratings::ActiveModel>, DbErr> =
                        entity::ratings::Entity::insert(ratings_table)
                            .exec(connection)
                            .await;

                    match ratings_insert {
                        Ok(ratings) => {
                            let reply_table = entity::reply::ActiveModel {
                                user_id: Set(authenticated.user_id),
                                buzz_id: Set(reply.buzz_id.parse().unwrap()),
                                reply_content: Set(reply.reply_content),
                                buzz_words: Set(reply.buzz_words),
                                mentioned_users: Set(reply.mentioned_users),
                                ratings_id: Set(Some(ratings.last_insert_id)),
                                created_at: Set(chrono::DateTime::from(chrono::Utc::now())),
                                ..Default::default()
                            };

                            let reply_insert = reply_table.insert(connection).await;

                            match reply_insert {
                                Ok(reply) => Ok(schemas::reply::ReplyResult {
                                    id: reply.id.to_string(),
                                    user_id: reply.user_id.to_string(),
                                    buzz_id: reply.buzz_id.to_string(),
                                    reply_content: reply.reply_content,
                                    buzz_words: reply.buzz_words,
                                    mentioned_users: reply.mentioned_users,
                                    ratings_id: Some(reply.ratings_id.unwrap_or(-1).to_string()),
                                    created_at: reply.created_at,
                                }),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                    }
                } else {
                    Err(FieldError::new(
                        "Cant create reply on behalf of other users",
                        juniper::Value::Null,
                    ))
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "delete reply")]
    async fn delete_reply(jwt: String, reply_id: String, context: &Context) -> FieldResult<bool> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                let get_reply = entity::reply::Entity::find_by_id(reply_id.parse::<i64>().unwrap())
                    .one(connection)
                    .await;
                match get_reply {
                    Ok(reply) => match reply {
                        Some(reply) => {
                            if reply.user_id.to_string() == authenticated.user_id.to_string() {
                                let reply_delete = entity::reply::Entity::delete_by_id(
                                    reply_id.parse::<i64>().unwrap(),
                                )
                                .exec(connection)
                                .await;
                                match reply_delete {
                                    Ok(_) => Ok(true),
                                    Err(e) => {
                                        Err(FieldError::new(e.to_string(), juniper::Value::Null))
                                    }
                                }
                            } else {
                                Err(FieldError::new(
                                    "Cant delete reply on behalf of other users",
                                    juniper::Value::Null,
                                ))
                            }
                        }
                        None => Err(FieldError::new("Reply not found", juniper::Value::Null)),
                    },
                    Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "upvote buzz/reply")]
    async fn upvote(
        jwt: String,
        ratings_id: String,
        context: &Context,
    ) -> FieldResult<schemas::ratings::UpvoteResponse> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        return match authentication {
            Authenticated(authenticated) => {
                let get_ratings =
                    entity::ratings::Entity::find_by_id(ratings_id.parse::<i64>().unwrap())
                        .one(connection)
                        .await;
                match get_ratings {
                    Ok(upvote) => match upvote {
                        Some(upvote) => {
                            let mut ratings_table: entity::ratings::ActiveModel = upvote.into();
                            let users = ratings_table
                                .get(entity::ratings::Column::UpvotedBy)
                                .into_value()
                                .unwrap()
                                .unwrap::<Option<String>>();

                            let mut finally_is_it_upvote: bool = false;

                            let mut add_user_to_upvoted_by =
                                |mut user_details: std::collections::HashSet<String>,
                                 value: i64| {
                                    if value == 1 {
                                        user_details.insert(authenticated.user_id.to_string());
                                        finally_is_it_upvote = true;
                                    } else {
                                        user_details.remove(&authenticated.user_id.to_string());
                                        finally_is_it_upvote = false;
                                    }

                                    ratings_table.upvotes = Set(Some(
                                        ratings_table
                                            .get(entity::ratings::Column::Upvotes)
                                            .into_value()
                                            .unwrap()
                                            .unwrap::<i64>()
                                            + value,
                                    ));
                                    ratings_table.upvoted_by =
                                        Set(Some(convert_set_to_string(user_details)));
                                };
                            match users {
                                Some(users) => {
                                    let users_set: std::collections::HashSet<String> =
                                        convert_string_to_set(users);

                                    if users_set.contains(&authenticated.user_id.to_string()) {
                                        add_user_to_upvoted_by(users_set, -1);
                                    } else {
                                        add_user_to_upvoted_by(users_set, 1);
                                    }
                                }

                                None => {
                                    let users_set: std::collections::HashSet<String> =
                                        std::collections::HashSet::new();

                                    add_user_to_upvoted_by(users_set, 1);
                                }
                            }

                            let ratings_update = ratings_table.update(connection).await;
                            match ratings_update {
                                Ok(_) => Ok(schemas::ratings::UpvoteResponse {
                                    is_upvoted: finally_is_it_upvote,
                                    id: ratings_id,
                                }),
                                Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                            }
                        }
                        None => Err(FieldError::new("Upvote not found", juniper::Value::Null)),
                    },
                    Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                }
            }
            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }

    #[graphql(description = "upvote buzz/reply")]
    async fn change_follow_user(
        jwt: String,
        follow_id: String,
        context: &Context,
    ) -> FieldResult<schemas::users::FollowResponse> {
        let connection = &context.connection;
        let authentication = authenticate(jwt).await;

        let change_following_table = |mut following_table: entity::users::ActiveModel,
                                      follow: bool| {
            let followers_list = following_table
                .get(entity::users::Column::Followers)
                .into_value()
                .unwrap()
                .unwrap::<Option<String>>();

            let mut followers_set: std::collections::HashSet<String> =
                convert_string_to_set(followers_list.unwrap_or_else(|| "".to_string()));

            if follow {
                followers_set.insert(follow_id.clone());
                following_table.followers = Set(Some(convert_set_to_string(followers_set)));
            } else {
                followers_set.remove(&follow_id);
                following_table.followers = Set(Some(convert_set_to_string(followers_set)));
            }
            following_table
        };

        return match authentication {
            Authenticated(authenticated) => {
                let get_follower = entity::users::Entity::find_by_id(authenticated.user_id)
                    .one(connection)
                    .await;

                let get_following =
                    entity::users::Entity::find_by_id(follow_id.parse::<i64>().unwrap())
                        .one(connection)
                        .await;

                match get_follower {
                    Ok(follower) => match follower {
                        Some(follower) => match get_following {
                            Ok(following) => match following {
                                Some(following) => {
                                    let mut follower_table: entity::users::ActiveModel =
                                        follower.into();
                                    let mut following_table: entity::users::ActiveModel =
                                        following.into();
                                    let following_list = follower_table
                                        .get(entity::users::Column::Following)
                                        .into_value()
                                        .unwrap()
                                        .unwrap::<Option<String>>();

                                    let mut following_list_set: std::collections::HashSet<String> =
                                        convert_string_to_set(
                                            following_list.unwrap_or_else(|| "".to_string()),
                                        );
                                    if following_list_set.contains(&follow_id) {
                                        following_list_set.remove(&follow_id);
                                        following_table =
                                            change_following_table(following_table, false);
                                    } else {
                                        following_list_set.insert(follow_id.clone());
                                        following_table =
                                            change_following_table(following_table, true);
                                    }
                                    follower_table.following = Set(Some(convert_set_to_string(
                                        following_list_set.clone(),
                                    )));
                                    let follower_update = follower_table.update(connection).await;
                                    let following_update = following_table.update(connection).await;
                                    match follower_update {
                                        Ok(_) => match following_update {
                                            Ok(following_model) => {
                                                Ok(schemas::users::FollowResponse {
                                                    following_id: following_model.id.to_string(),
                                                    is_following: following_list_set
                                                        .contains(&follow_id),
                                                })
                                            }
                                            Err(e) => Err(FieldError::new(
                                                e.to_string(),
                                                juniper::Value::Null,
                                            )),
                                        },
                                        Err(e) => Err(FieldError::new(
                                            e.to_string(),
                                            juniper::Value::Null,
                                        )),
                                    }
                                }

                                None => Err(FieldError::new(
                                    "Following user not found",
                                    juniper::Value::Null,
                                )),
                            },
                            Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                        },
                        None => Err(FieldError::new(
                            "Follower user not found",
                            juniper::Value::Null,
                        )),
                    },
                    Err(e) => Err(FieldError::new(e.to_string(), juniper::Value::Null)),
                }
            }

            Unauthenticated => Err(FieldError::new(
                "Authentication Failed",
                juniper::Value::Null,
            )),
        };
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
