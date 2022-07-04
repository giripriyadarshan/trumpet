#[derive(GraphQLInputObject)]
#[graphql(description = "upvote a buzz/reply")]
pub struct Upvote {
    pub is_buzz: bool,
    pub is_downvote: bool,
    pub id: String,
}

#[derive(GraphQLObject)]
pub struct RatingsDetails {
    pub id: i32,
    pub is_buzz: bool,
    pub module_id: i32,
    pub upvotes: String,
    pub views: String,
}
