#[derive(GraphQLInputObject)]
#[graphql(description = "Check if authenticated")]
pub struct AuthModify {
    pub username: String,
    pub email: String,
    pub contact_number: Option<String>,
    pub password: String,
}

#[derive(GraphQLObject)]
pub struct AuthResponse {
    pub username: String,
    pub email: String,
    pub contact_number: Option<String>,
}
