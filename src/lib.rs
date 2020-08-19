pub mod users;

#[macro_use] extern crate error_chain;

error_chain! {
    errors {
        TimelineError(msg: &'static str) {
            description("timeline error")
            display("timeline error: {}", msg)
        }
    }
    foreign_links {
        Http(reqwest::Error);
    }
}

use serde::Deserialize;
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;

/// Represents a response that might contain a cursor that can go forwards or backwards.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPage<'a, T: std::marker::Sync + DeserializeOwned + Send> {
    #[serde(skip)]
    pub base_url: Option<&'a String>,
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: T
}

/// Allows you to create a instance of a timeline from the current page.
/// The timeline lifetime created is based off the page's lifetime.
impl<'a, T: std::marker::Sync + DeserializeOwned + Send> CursorPage<'a, T> {
    pub fn timeline(&self) -> impl AsyncTimeline<Output = CursorPage<T>>  + '_ {
        CursorAsyncTimeline {
            base_url: self.base_url.as_ref().unwrap(),
            current: self
        }
    }
}

/// Represents an asynchronous page timeline. You can go back and forth within a timeline, however
/// the current state of the timeline is immutable. If you go forth, the current timeline remains the same; if you want to continue from this point, it requires
/// a new timeline.
#[async_trait]
pub trait AsyncTimeline {
    type Output;

    /// Gets the current reference in the timeline.
    fn current(&self) -> &Self::Output;

    /// Peeks forward in a timeline.
    async fn forward(&self) -> Result<Self::Output>;

    /// Peeks backward in a timeline.
    async fn backwards(&self) -> Result<Self::Output>;

}

/// Asynchronous cursor timeline implementation. T must be a serde deserializable type.
/// Base url and the current reference lives as long as the timeline.
struct CursorAsyncTimeline<'a, T: std::marker::Sync + DeserializeOwned + Send> {
    base_url: &'a String,
    current: &'a CursorPage<'a, T>
}

/// Implementation of CursorAsyncTimeline
/// As implemented according to the specs of [AsyncTimeline].
#[async_trait]
impl<'a, T: std::marker::Sync + DeserializeOwned + Send> AsyncTimeline for CursorAsyncTimeline<'a, T> {
    type Output = CursorPage<'a, T>;

    fn current(&self) -> &Self::Output {
        self.current
    }

    async fn forward(&self) -> Result<Self::Output> {
        Ok(point(self.base_url, self.current.next_page_cursor.as_ref().ok_or_else(|| ErrorKind::TimelineError("No next page cursor"))?).await?)
    }

    async fn backwards(&self) -> Result<Self::Output> {
        Ok(point(self.base_url, self.current.previous_page_cursor.as_ref().ok_or_else(|| ErrorKind::TimelineError("No previous page cursor"))?).await?)
    }
}

/// Points to a cursor id and retrieves the cursor page from the specified endpoint.
pub async fn point<'a, T: std::marker::Sync + serde::de::DeserializeOwned + Send>(base_url: &'a String, cursor_id: &'a String) -> Result<CursorPage<'a, T>> {
    let client = Client::new();
    let mut resp = client
        .get(base_url)
        .query(&[("limit", "100"), ("cursor", cursor_id)])
        .send()
        .await?
        .json::<CursorPage<T>>()
        .await?;
    resp.base_url = Some(&base_url);
    Ok(resp)
}

