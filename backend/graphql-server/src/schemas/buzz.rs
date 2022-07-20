use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(GraphQLInputObject)]
#[graphql(description = "Create the buzz")]
pub struct BuzzInput {
    pub user_id: String,
    pub description: String,
    pub image_link: Option<String>,
    pub video_link: Option<String>,
    pub buzz_words: Option<String>,
    pub mentioned_users: Option<String>,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Get all buzzes")]
pub struct GetAllBuzzInput {
    pub page_size: i32,
    pub page_number: i32,
    pub user_id: Option<String>,
}

#[derive(GraphQLObject)]
pub struct BuzzResult {
    pub id: String,
    pub user_id: String,
    pub description: String,
    pub image_link: Option<String>,
    pub video_link: Option<String>,
    pub buzz_words: Option<String>,
    pub mentioned_users: Option<String>,
    pub ratings_id: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(GraphQLObject)]
pub struct AllBuzzResult {
    pub buzzes: Vec<BuzzResult>,
    pub total_buzzes: i32,
    pub total_pages: i32,
    pub page_number: i32,
    pub page_size: i32,
}
