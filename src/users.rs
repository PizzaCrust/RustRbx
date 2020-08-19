use serde::{Deserialize, Serialize};

/// Represents a user returned by a query's response.
#[derive(Deserialize, Serialize, Debug)]
pub struct UserQuery {
    pub id: u64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String
}

/// Represents a user returned by a detailed query regarding a specified user.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Description of the user
    pub desc: String,
    /// Iso instant date of the creation of the account
    pub created: String,
    pub is_banned: bool,
    pub id: u64,
    pub name: String,
    pub display_name: String
}

