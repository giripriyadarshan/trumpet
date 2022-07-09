use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(GraphQLInputObject)]
#[graphql(description = "Create reply")]
pub struct ReplyInput {
    pub user_id: String,
    pub buzz_id: String,
    pub reply_content: String,
    pub buzz_words: Option<String>,
    pub mentioned_users: Option<String>,
}

#[derive(GraphQLObject)]
pub struct ReplyResult {
    pub id: String,
    pub user_id: String,
    pub buzz_id: String,
    pub reply_content: String,
    pub buzz_words: Option<String>,
    pub mentioned_users: Option<String>,
    pub ratings_id: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}
