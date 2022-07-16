#[derive(GraphQLInputObject)]
pub struct UserModify {
    pub full_name: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub location_or_region: Option<String>,
}

#[derive(GraphQLObject)]
pub struct UserDetails {
    pub id: i32,
    pub auth_id: Option<i32>,
    pub full_name: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub location_or_region: Option<String>,
}

#[derive(GraphQLObject)]
pub struct FollowResponse {
    pub following_id: String,
    pub is_following: bool,
}
