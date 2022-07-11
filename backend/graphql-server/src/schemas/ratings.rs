#[derive(GraphQLInputObject)]
#[graphql(description = "upvote a buzz/reply")]
pub struct UpvoteInput {
    pub id: String,
}

#[derive(GraphQLObject)]
pub struct RatingsDetails {
    pub id: i32,
    pub upvotes: String,
    pub views: String,
}

#[derive(GraphQLObject)]
pub struct UpvoteResponse {
    pub id: String,
    pub is_upvoted: bool,
}
