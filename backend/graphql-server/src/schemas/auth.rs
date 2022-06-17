#[derive(GraphQLInputObject)]
#[graphql(description = "Check if authenticated")]
pub struct AuthModify {
    pub username: String,
    pub email: String,
    pub contact_number: Option<String>,
    pub password: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Send Authentication Token received from auth server")]
pub struct AuthToken {
    #[graphql(description = "Authentication Token")]
    pub token: String,
}
