use anyhow::Context;

use crate::models;

const DEFAULT_API_BASE: &str = "https://tempus2.xyz/api/v0";
const MAPS_DETAILED_LIST: &str = "/maps/detailedList";

#[derive(Debug, Clone)]
pub struct TempusClient {
    inner: reqwest::Client,
    base: String,
}

impl TempusClient {
    pub fn new() -> Self {
        Self::new_with_base(DEFAULT_API_BASE)
    }

    pub fn new_with_base(base: impl Into<String>) -> Self {
        let reqwest = reqwest::Client::new();
        Self {
            inner: reqwest,
            base: base.into(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_maps(&self) -> anyhow::Result<Vec<models::Map>> {
        let response = self
            .inner
            .get(self.base.clone() + MAPS_DETAILED_LIST)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("http status: {status}");
        }

        let body = response.text().await?;
        let maps: Vec<models::Map> = serde_json::from_str(&body).with_context(|| {
            format!("failed to parse response: {}", &body[..body.len().min(500)])
        })?;
        Ok(maps)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_map_records(&self, map_id: i32) -> anyhow::Result<models::MapRecordsList> {
        let url = format!("/maps/id/{map_id}/zones/typeindex/map/1/records/list?limit=0");
        let response = self.inner.get(self.base.clone() + &url).send().await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("http status: {status}");
        }

        let body = response.text().await?;
        let records_list = serde_json::from_str(&body).with_context(|| {
            format!("failed to parse response: {}", &body[..body.len().min(500)])
        })?;
        Ok(records_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn get_maps_mocked() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(MAPS_DETAILED_LIST))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(include_str!("../test_data/maps.json"), "application/json"),
            )
            .mount(&server)
            .await;

        let client = TempusClient::new_with_base(&server.uri());
        let maps = client.get_maps().await.unwrap();

        assert_eq!(maps.len(), 775);
    }

    #[tokio::test]
    async fn get_records_mocked() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/maps/id/136/zones/typeindex/map/1/records/list"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                include_str!("../test_data/map_records_136.json"),
                "application/json",
            ))
            .mount(&server)
            .await;

        let client = TempusClient::new_with_base(&server.uri());
        let records_list = client.get_map_records(136).await.unwrap();

        assert_eq!(records_list.records.soldier.len(), 50);
        assert_eq!(records_list.records.demoman.len(), 50);
    }
}
