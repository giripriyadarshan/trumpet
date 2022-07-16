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
