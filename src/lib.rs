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
pub struct PageCursor<T> {
    #[serde(skip)]
    pub base_url: String,
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: T
}

/// Allows you to create a instance of a timeline from the current page.
/// Takes ownership of the cursor page and returns a timeline.
impl<T: std::marker::Sync + DeserializeOwned + Send> PageCursor<T> {
    pub fn timeline(self) -> CursorAsyncTimeline<T> {
        CursorAsyncTimeline {
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
    fn current(&mut self) -> &mut Self::Output;

    /// Peeks forward in a timeline.
    async fn forward(&self) -> Result<Self::Output>;

    /// Peeks backward in a timeline.
    async fn backwards(&self) -> Result<Self::Output>;

}

/// Asynchronous cursor timeline implementation. T must be a serde deserializable type.
/// Must give ownership of the cursor, can go forwards or backwards and can access current cursor. Timeline is not mutable, however reference is.
pub struct CursorAsyncTimeline<T: std::marker::Sync + DeserializeOwned + Send> {
    current: PageCursor<T>
}

/// Implementation of CursorAsyncTimeline
/// As implemented according to the specs of [AsyncTimeline].
#[async_trait]
impl<T: std::marker::Sync + DeserializeOwned + Send> AsyncTimeline for CursorAsyncTimeline<T> {
    type Output = PageCursor<T>;

    fn current(&mut self) -> &mut Self::Output {
        &mut self.current
    }

    async fn forward(&self) -> Result<Self::Output> {
        Ok(point(&self.current.base_url, self.current.next_page_cursor.as_ref().ok_or_else(|| ErrorKind::TimelineError("No next page cursor"))?).await?)
    }

    async fn backwards(&self) -> Result<Self::Output> {
        Ok(point(&self.current.base_url, self.current.previous_page_cursor.as_ref().ok_or_else(|| ErrorKind::TimelineError("No previous page cursor"))?).await?)
    }
}

/// Points to a cursor id and retrieves the cursor page from the specified endpoint.
pub async fn point<'a, T: std::marker::Sync + serde::de::DeserializeOwned + Send>(base_url: &'a String, cursor_id: &'a String) -> Result<PageCursor<T>> {
    let client = Client::new();
    let mut resp = client
        .get(base_url)
        .query(&[("limit", "100"), ("cursor", cursor_id)])
        .send()
        .await?
        .json::<PageCursor<T>>()
        .await?;
    resp.base_url = base_url.to_string();
    Ok(resp)
}

/// Iterate over the contents of the current cursor.
/// Will cache more if necessary. Note that has_remaining may have extra overhead.
/// Can request with capacity so it will cache up to the specified amount to access the data between the amount specified faster.
pub struct AsyncItemIterator<T: std::marker::Sync + DeserializeOwned + Send> {
    timeline: CursorAsyncTimeline<Vec<T>>
}

impl<T: std::marker::Sync + DeserializeOwned + Send> AsyncItemIterator<T> {

    pub fn current(self) -> CursorAsyncTimeline<Vec<T>> {
        self.timeline
    }

    pub fn new(timeline: CursorAsyncTimeline<Vec<T>>) -> AsyncItemIterator<T> {
        AsyncItemIterator {
            timeline
        }
    }

    /// Creates a new instance of [AsyncItemIterator] but will cache the capacity size initially.
    /// If the capacity is over the amount of actual data, it will be cut off at that point.
    pub async fn with_capacity(cursor: PageCursor<Vec<T>>, capacity: &mut u32) -> Result<AsyncItemIterator<T>> {
        let mut current_cursor = cursor;
        let mut vec: Vec<T> = Vec::new();
        while *capacity > 0 {
            if *capacity > 100 {
                *capacity -= 100;
            } else {
                *capacity = 0;
            }
            vec.append(&mut current_cursor.data);
            let timeline = current_cursor.timeline();
            match timeline.forward().await {
                Ok(e) => {
                    current_cursor = e;
                }
                _ => {
                    current_cursor = timeline.current;
                    break
                }
            }
        }
        current_cursor.data.append(&mut vec);
        Ok(Self::new(current_cursor.timeline()))
    }

    pub async fn next(&mut self) -> crate::Result<T> {
        if self.timeline.current.data.is_empty() {
            self.timeline = self.timeline.forward().await?.timeline()
        }
        Ok(self.timeline.current.data.remove(0))
    }

    pub async fn has_remaining(&self) -> bool {
        if !self.timeline.current.data.is_empty() {
            return true;
        }
        let forward = self.timeline.forward().await;
        return match forward {
            Ok(v) => {
                !v.data.is_empty()
            }
            Err(_) => false
        }
    }

}