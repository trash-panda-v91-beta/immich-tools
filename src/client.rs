use anyhow::{Context, Result};
use futures_util::TryStreamExt;
use reqwest::{
    multipart::{Form, Part},
    Client, StatusCode,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio_util::io::StreamReader;

#[derive(Clone)]
pub struct ImmichClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub original_file_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    pub assets: AssetPage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPage {
    pub items: Vec<Asset>,
    pub next_page: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRequest {
    is_favorite: bool,
    size: u32,
    page: u32,
}

impl ImmichClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(900))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn search_favorites(&self, page: u32) -> Result<SearchResults> {
        let body = SearchRequest {
            is_favorite: true,
            size: 1000,
            page,
        };
        let resp = self
            .client
            .post(format!("{}/api/search/metadata", self.base_url))
            .header("x-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("search request failed")?
            .error_for_status()
            .context("search returned error status")?;
        resp.json()
            .await
            .context("failed to deserialize search response")
    }

    pub async fn download_asset(&self, id: &str, dest: &Path) -> Result<()> {
        let resp = self
            .client
            .get(format!("{}/api/assets/{}/original", self.base_url, id))
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("download request failed")?
            .error_for_status()
            .context("download returned error status")?;

        let tmp = dest.with_extension("tmp");
        let stream = resp.bytes_stream().map_err(std::io::Error::other);
        let mut reader = StreamReader::new(stream);
        let mut file = fs::File::create(&tmp)
            .await
            .context("failed to create tmp file")?;
        tokio::io::copy(&mut reader, &mut file)
            .await
            .context("failed to stream download to file")?;
        drop(file);
        fs::rename(&tmp, dest)
            .await
            .context("failed to rename tmp to dest")?;
        Ok(())
    }

    /// Returns the asset ID if upload succeeded, None if asset already exists.
    pub async fn upload_asset_bytes(
        &self,
        filename: &str,
        dedup_key: &str,
        bytes: Vec<u8>,
    ) -> Result<Option<String>> {
        let mime = mime_guess::from_path(filename)
            .first_or_octet_stream()
            .to_string();

        let part = Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str(&mime)?;
        let form = Form::new()
            .text("deviceAssetId", dedup_key.to_string())
            .text("deviceId", "immich-tools")
            .text("fileCreatedAt", chrono::Utc::now().to_rfc3339())
            .text("fileModifiedAt", chrono::Utc::now().to_rfc3339())
            .part("assetData", part);

        let resp = self
            .client
            .post(format!("{}/api/assets", self.base_url))
            .header("x-api-key", &self.api_key)
            .multipart(form)
            .send()
            .await
            .context("upload request failed")?;

        if resp.status() == StatusCode::OK {
            // 200 means duplicate
            return Ok(None);
        }
        resp.error_for_status_ref()
            .context("upload returned error status")?;
        let body: serde_json::Value = resp
            .json()
            .await
            .context("failed to deserialize upload response")?;
        let id = body["id"]
            .as_str()
            .context("upload response missing id")?
            .to_string();
        Ok(Some(id))
    }
}
