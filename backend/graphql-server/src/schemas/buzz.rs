#[derive(GraphQLInputObject)]
#[graphql(description = "Create the buzz")]
pub struct BuzzInput {
    pub user_id: String,
    pub description: String,
    pub image_link: Option<String>,
    pub video_link: Option<String>,
    pub buzz_words: Option<Vec<String>>,
    pub mentioned_users: Option<Vec<String>>,
}

#[derive(GraphQLObject)]
pub struct BuzzResult {
    pub id: String,
    pub user_id: String,
    pub description: String,
    pub image_link: Option<String>,
    pub video_link: Option<String>,
    pub buzz_words: Option<Vec<String>>,
    pub mentioned_users: Option<Vec<String>>,
}