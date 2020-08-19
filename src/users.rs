use serde::{Deserialize, Serialize};
use crate::PageCursor;
use reqwest::Client;
use crate::Result;

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
    pub description: String,
    /// Iso instant date of the creation of the account
    pub created: String,
    pub is_banned: bool,
    pub id: u64,
    pub name: String,
    pub display_name: String
}

const BASE_URL: &str = "https://users.roblox.com";

pub async fn search(keyword: String) -> Result<PageCursor<Vec<UserQuery>>> {
    let client = Client::new();
    let resp = client
        .get(&format!("{}/v1/users/search", BASE_URL).to_string())
        .query(&[("limit", "100"),("keyword", &keyword)])
        .send()
        .await?;
    let url = resp.url().to_string();
    let mut json = resp.json::<PageCursor<Vec<UserQuery>>>().await?;
    json.base_url = url;
    Ok(json)
}

pub async fn get(id: u64) -> Result<User> {
    let client = Client::new();
    Ok(client
        .get(&format!("{}/v1/users/{}", BASE_URL, id))
        .send()
        .await?
        .json()
        .await?)
}