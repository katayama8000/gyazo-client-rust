use reqwest::multipart::{Form, Part};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for the Gyazo API client
#[derive(Error, Debug)]
pub enum GyazoError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("API error: {status}, message: {message}")]
    ApiError { status: StatusCode, message: String },
    #[error("Unexpected error: {0}")]
    Other(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Gyazo API client
pub struct GyazoClient {
    client: Client,
    access_token: String,
}

impl GyazoClient {
    /// Create a new GyazoClient instance
    pub fn new(access_token: String) -> Self {
        GyazoClient {
            client: Client::new(),
            access_token,
        }
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        method: reqwest::Method,
        form: Option<Form>,
    ) -> Result<T, GyazoError> {
        let mut request = self
            .client
            .request(method, url)
            .bearer_auth(&self.access_token);

        if let Some(form) = form {
            request = request.multipart(form);
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => {
                Ok(response.json().await?)
            }
            status => {
                let message = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(GyazoError::ApiError { status, message })
            }
        }
    }

    /// Get an image by its ID
    pub async fn get_image(&self, image_id: &str) -> Result<GyazoImageResponse, GyazoError> {
        let url = format!("https://api.gyazo.com/api/images/{}", image_id);
        self.request(&url, reqwest::Method::GET, None).await
    }

    /// Get a list of images
    pub async fn list_images(&self) -> Result<Vec<GyazoImageResponse>, GyazoError> {
        let url = "https://api.gyazo.com/api/images".to_string();
        self.request(&url, reqwest::Method::GET, None).await
    }

    /// Upload an image
    pub async fn upload_image(
        &self,
        param: UploadParams,
    ) -> Result<UploadImageResponse, GyazoError> {
        let url = "https://upload.gyazo.com/api/upload".to_string();
        let mut form = Form::new().part(
            "imagedata",
            Part::bytes(param.imagedata.clone()).file_name("image.png"),
        );

        for (key, value) in param.into_form_params() {
            form = form.text(key, value);
        }

        self.request(&url, reqwest::Method::POST, Some(form)).await
    }

    /// Delete an image by its ID
    pub async fn delete_image(&self, image_id: &str) -> Result<DeleteImageResponse, GyazoError> {
        let url = format!("https://api.gyazo.com/api/images/{}", image_id);
        self.request(&url, reqwest::Method::DELETE, None).await
    }
}

/// Image response from Gyazo API
#[derive(Debug, Deserialize)]
pub struct GyazoImageResponse {
    pub image_id: String,
    pub permalink_url: Option<String>,
    pub thumb_url: Option<String>,
    #[serde(rename = "type")]
    pub image_type: String,
    pub created_at: String,
    pub metadata: ImageMetadata,
    pub ocr: Option<ImageOcr>,
}

#[derive(Debug, Deserialize)]
pub struct ImageMetadata {
    pub app: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub desc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageOcr {
    pub locale: String,
    pub description: String,
}

/// Response after uploading an image
#[derive(Debug, Deserialize)]
pub struct UploadImageResponse {
    pub image_id: String,
    pub permalink_url: String,
    pub thumb_url: String,
    pub url: String,
    #[serde(rename = "type")]
    pub image_type: String,
}

/// Response after deleting an image
#[derive(Debug, Deserialize)]
pub struct DeleteImageResponse {
    pub image_id: String,
    #[serde(rename = "type")]
    pub image_type: String,
}

/// Parameters for uploading an image
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct UploadParams {
    pub imagedata: Vec<u8>,
    pub access_policy: Option<String>,
    pub metadata_is_public: Option<String>,
    pub referer_url: Option<String>,
    pub app: Option<String>,
    pub title: Option<String>,
    pub desc: Option<String>,
    pub created_at: Option<String>,
    pub collection_id: Option<String>,
}

impl UploadParams {
    fn into_form_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(access_policy) = &self.access_policy {
            params.push(("access_policy".to_string(), access_policy.clone()));
        }
        params.push((
            "metadata_is_public".to_string(),
            self.metadata_is_public
                .clone()
                .unwrap_or_else(|| "true".to_string()),
        ));
        if let Some(referer_url) = &self.referer_url {
            params.push(("referer_url".to_string(), referer_url.clone()));
        }
        if let Some(app) = &self.app {
            params.push(("app".to_string(), app.clone()));
        }
        if let Some(title) = &self.title {
            params.push(("title".to_string(), title.clone()));
        }
        if let Some(desc) = &self.desc {
            params.push(("desc".to_string(), desc.clone()));
        }
        if let Some(created_at) = &self.created_at {
            params.push(("created_at".to_string(), created_at.to_string()));
        }
        if let Some(collection_id) = &self.collection_id {
            params.push(("collection_id".to_string(), collection_id.clone()));
        }
        params
    }
}

/// Builder for UploadParams
pub struct UploadParamsBuilder {
    imagedata: Vec<u8>,
    access_policy: Option<String>,
    metadata_is_public: Option<String>,
    referer_url: Option<String>,
    app: Option<String>,
    title: Option<String>,
    desc: Option<String>,
    created_at: Option<String>,
    collection_id: Option<String>,
}

impl UploadParamsBuilder {
    pub fn new(imagedata: Vec<u8>) -> Self {
        Self {
            imagedata,
            access_policy: None,
            metadata_is_public: None,
            referer_url: None,
            app: None,
            title: None,
            desc: None,
            created_at: None,
            collection_id: None,
        }
    }

    pub fn access_policy(mut self, access_policy: impl Into<String>) -> Result<Self, GyazoError> {
        let access_policy = access_policy.into();
        if access_policy != "anyone" && access_policy != "only_me" {
            return Err(GyazoError::InvalidInput(
                "access_policy must be 'anyone' or 'only_me'".to_string(),
            ));
        }
        self.access_policy = Some(access_policy);
        Ok(self)
    }

    pub fn metadata_is_public(
        mut self,
        metadata_is_public: impl Into<String>,
    ) -> Result<Self, GyazoError> {
        let metadata_is_public = metadata_is_public.into();
        if metadata_is_public != "true" && metadata_is_public != "false" {
            return Err(GyazoError::InvalidInput(
                "metadata_is_public must be 'true' or 'false'".to_string(),
            ));
        }
        self.metadata_is_public = Some(metadata_is_public);
        Ok(self)
    }

    pub fn referer_url(mut self, referer_url: impl Into<String>) -> Self {
        self.referer_url = Some(referer_url.into());
        self
    }

    pub fn app(mut self, app: impl Into<String>) -> Self {
        self.app = Some(app.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = Some(desc.into());
        self
    }

    pub fn created_at(mut self, created_at: impl Into<String>) -> Self {
        self.created_at = Some(created_at.into());
        self
    }

    pub fn collection_id(mut self, collection_id: impl Into<String>) -> Self {
        self.collection_id = Some(collection_id.into());
        self
    }

    pub fn build(self) -> UploadParams {
        UploadParams {
            imagedata: self.imagedata,
            access_policy: self.access_policy,
            metadata_is_public: self.metadata_is_public,
            referer_url: self.referer_url,
            app: self.app,
            title: self.title,
            desc: self.desc,
            created_at: self.created_at,
            collection_id: self.collection_id,
        }
    }
}
