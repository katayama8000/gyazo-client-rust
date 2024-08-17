use reqwest::multipart::Form;
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_BASE_URL: &str = "https://api.gyazo.com";
const DEFAULT_UPLOAD_URL: &str = "https://upload.gyazo.com";

/// Error types for the Gyazo API client
#[derive(Error, Debug)]
pub enum GyazoError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Bad Request: Invalid request parameters")]
    BadRequest,
    #[error("Unauthorized: Authentication required")]
    Unauthorized,
    #[error("Forbidden: Access denied")]
    Forbidden,
    #[error("Not Found")]
    NotFound,
    #[error("Unprocessable Entity: Server cannot process the request")]
    UnprocessableEntity,
    #[error("Too Many Requests: Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Internal Server Error: Unexpected error occurred")]
    InternalServerError,
    #[error("API error: {status}, message: {message}")]
    ApiError { status: StatusCode, message: String },
    #[error("Unexpected error: {0}")]
    Other(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid url: {0}")]
    InvalidUrl(String),
}

/// Gyazo API client
#[derive(Clone, Debug)]
pub struct GyazoClient {
    client: Client,
    access_token: String,
    base_url: Url,
    upload_url: Url,
}

#[derive(Default, Clone, Debug)]
pub struct GyazoClientOptions {
    pub access_token: String,
    pub base_url: Option<String>,
    pub upload_url: Option<String>,
}

impl GyazoClient {
    /// Create a new GyazoClient instance
    pub fn new(options: GyazoClientOptions) -> Self {
        let base_url = options
            .base_url
            .map(|url| Url::parse(&url).expect("base_url must be a valid URL"))
            .unwrap_or_else(|| Url::parse(DEFAULT_BASE_URL).expect("base_url must be a valid URL"));
        let upload_url = options
            .upload_url
            .map(|url| Url::parse(&url).expect("upload_url must be a valid URL"))
            .unwrap_or_else(|| {
                Url::parse(DEFAULT_UPLOAD_URL).expect("upload_url must be a valid URL")
            });
        GyazoClient {
            client: Client::new(),
            access_token: options.access_token,
            base_url,
            upload_url,
        }
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        method: reqwest::Method,
        form: Option<Form>,
    ) -> Result<T, GyazoError> {
        let url = if path == "/api/upload" {
            self.upload_url
                .join(path)
                .expect("path must be a valid URL")
        } else {
            self.base_url.join(path).expect("path must be a valid URL")
        };
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
            StatusCode::BAD_REQUEST => Err(GyazoError::BadRequest),
            StatusCode::UNAUTHORIZED => Err(GyazoError::Unauthorized),
            StatusCode::FORBIDDEN => Err(GyazoError::Forbidden),
            StatusCode::NOT_FOUND => Err(GyazoError::NotFound),
            StatusCode::UNPROCESSABLE_ENTITY => Err(GyazoError::UnprocessableEntity),
            StatusCode::TOO_MANY_REQUESTS => Err(GyazoError::RateLimitExceeded),
            StatusCode::INTERNAL_SERVER_ERROR => Err(GyazoError::InternalServerError),
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
        let path = format!("/api/images/{}", image_id);
        self.request(&path, reqwest::Method::GET, None).await
    }

    /// Get a list of images
    pub async fn list_images(&self) -> Result<Vec<GyazoImageResponse>, GyazoError> {
        let path = "/api/images".to_string();
        self.request(&path, reqwest::Method::GET, None).await
    }

    /// Upload an image
    pub async fn upload_image(
        &self,
        param: UploadParams,
    ) -> Result<UploadImageResponse, GyazoError> {
        let path = "/api/upload";
        let form = param.into();
        self.request(path, reqwest::Method::POST, Some(form)).await
    }

    /// Delete an image by its ID
    pub async fn delete_image(&self, image_id: &str) -> Result<DeleteImageResponse, GyazoError> {
        let path = format!("/api/images/{}", image_id);
        self.request(&path, reqwest::Method::DELETE, None).await
    }

    /// Get oembed data for an image
    pub async fn get_oembed(&self, url: &str) -> Result<OembedResponse, GyazoError> {
        if !url.starts_with("https://gyazo.com/") {
            return Err(GyazoError::InvalidUrl(
                "URL must start with 'https://gyazo.com/'".to_string(),
            ));
        }
        let url = format!("/api/oembed?url={}", url);
        self.request(&url, reqwest::Method::GET, None).await
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
#[derive(Debug)]
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

impl Into<reqwest::multipart::Form> for UploadParams {
    fn into(self) -> reqwest::multipart::Form {
        let mut form = reqwest::multipart::Form::new().part(
            "imagedata",
            reqwest::multipart::Part::bytes(self.imagedata).file_name("image.png"),
        );
        form = form.text(
            "access_policy",
            self.access_policy.unwrap_or_else(|| "anyone".to_string()),
        );
        if let Some(metadata_is_public) = self.metadata_is_public {
            form = form.text("metadata_is_public", metadata_is_public);
        }
        if let Some(referer_url) = self.referer_url {
            form = form.text("referer_url", referer_url);
        }
        if let Some(app) = self.app {
            form = form.text("app", app);
        }
        if let Some(title) = self.title {
            form = form.text("title", title);
        }
        if let Some(desc) = self.desc {
            form = form.text("desc", desc);
        }
        if let Some(created_at) = self.created_at {
            form = form.text("created_at", created_at);
        }
        if let Some(collection_id) = self.collection_id {
            form = form.text("collection_id", collection_id);
        }
        form
    }
}

/// Builder for UploadParams
#[derive(Debug)]
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

    pub fn build(self) -> Result<UploadParams, GyazoError> {
        Ok(UploadParams {
            imagedata: self.imagedata,
            access_policy: self.access_policy,
            metadata_is_public: self.metadata_is_public,
            referer_url: self.referer_url,
            app: self.app,
            title: self.title,
            desc: self.desc,
            created_at: self.created_at,
            collection_id: self.collection_id,
        })
    }
}

/// Oembed response from Gyazo API
#[derive(Debug, Deserialize)]
pub struct OembedResponse {
    pub version: String,
    #[serde(rename = "type")]
    pub image_type: String,
    pub provider_name: String,
    pub provider_url: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;

    #[tokio::test]
    async fn test_get_image() -> anyhow::Result<()> {
        let mut server = mockito::Server::new_async().await;
        let mock_response = r#"
        {
            "image_id": "abc123",
            "permalink_url": "https://gyazo.com/abc123",
            "thumb_url": "https://thumb.gyazo.com/thumb/abc123",
            "type": "png",
            "created_at": "2024-08-10 12:00:00",
            "metadata": {
                "app": null,
                "title": null,
                "url": null,
                "desc": null
            },
            "ocr": null
        }
        "#;

        server
            .mock("GET", "/api/images/abc123")
            .match_header("Authorization", Matcher::Regex("Bearer .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = GyazoClient::new(GyazoClientOptions {
            access_token: "fake_token".to_string(),
            base_url: Some(server.url().to_string()),
            upload_url: None,
        });
        let result = client.get_image("abc123").await;

        assert!(result.is_ok());
        let image = result?;
        assert_eq!(image.image_id, "abc123");
        assert_eq!(
            image.permalink_url,
            Some("https://gyazo.com/abc123".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_list_images() -> anyhow::Result<()> {
        let mut server = mockito::Server::new_async().await;
        let mock_response = r#"
        [
            {
                "image_id": "abc123",
                "permalink_url": "https://gyazo.com/abc123",
                "thumb_url": "https://thumb.gyazo.com/thumb/abc123",
                "type": "png",
                "created_at": "2024-08-10 12:00:00",
                "metadata": {
                    "app": null,
                    "title": null,
                    "url": null,
                    "desc": null
                },
                "ocr": null
            }
        ]
        "#;

        server
            .mock("GET", "/api/images")
            .match_header("Authorization", Matcher::Regex("Bearer .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = GyazoClient::new(GyazoClientOptions {
            access_token: "fake_token".to_string(),
            base_url: Some(server.url().to_string()),
            upload_url: None,
        });

        let result = client.list_images().await;

        assert!(result.is_ok());
        let images = result?;
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].image_id, "abc123");
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_image() -> anyhow::Result<()> {
        let mut server = mockito::Server::new_async().await;
        let mock_response = r#"
    {
        "image_id": "abc123",
        "permalink_url": "https://gyazo.com/abc123",
        "thumb_url": "https://thumb.gyazo.com/thumb/abc123",
        "url": "https://i.gyazo.com/abc123.png",
        "type": "png"
    }
    "#;

        server
            .mock("POST", "/api/upload")
            .match_header("Authorization", Matcher::Regex("Bearer .+".to_string()))
            .match_body(Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = GyazoClient::new(GyazoClientOptions {
            access_token: "fake_token".to_string(),
            base_url: None,
            upload_url: Some(server.url().to_string()),
        });
        let params = UploadParamsBuilder::new(vec![0, 1, 2, 3])
            .title("test image")
            .build()?;
        let result = client.upload_image(params).await;

        assert!(result.is_ok());
        let image = result?;
        assert_eq!(image.image_id, "abc123");
        assert_eq!(image.permalink_url, "https://gyazo.com/abc123".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_image() -> anyhow::Result<()> {
        let mut server = mockito::Server::new_async().await;
        let mock_response = r#"
        {
            "image_id": "abc123",
            "type": "png"
        }
        "#;

        server
            .mock("DELETE", "/api/images/abc123")
            .match_header("Authorization", Matcher::Regex("Bearer .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = GyazoClient::new(GyazoClientOptions {
            access_token: "fake_token".to_string(),
            base_url: Some(server.url().to_string()),
            upload_url: None,
        });
        let result = client.delete_image("abc123").await;

        assert!(result.is_ok());
        let image = result?;
        assert_eq!(image.image_id, "abc123");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_oembed() -> anyhow::Result<()> {
        let mut server = mockito::Server::new_async().await;
        let mock_response = r#"
        {
            "version": "1.0",
            "type": "photo",
            "provider_name": "Gyazo",
            "provider_url": "https://gyazo.com",
            "url": "https://i.gyazo.com/abc123.png",
            "width": 400,
            "height": 300
        }
        "#;

        server
            .mock("GET", "/api/oembed?url=https://gyazo.com/abc123")
            .match_header("Authorization", Matcher::Regex("Bearer .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = GyazoClient::new(GyazoClientOptions {
            access_token: "fake_token".to_string(),
            base_url: Some(server.url().to_string()),
            upload_url: None,
        });
        let result = client.get_oembed("https://gyazo.com/abc123").await;

        assert!(result.is_ok());
        let oembed = result?;
        assert_eq!(oembed.version, "1.0");
        assert_eq!(oembed.image_type, "photo");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_oembed_invalid_url() -> anyhow::Result<()> {
        let client = GyazoClient::new(GyazoClientOptions {
            access_token: "fake_token".to_string(),
            ..Default::default()
        });
        let result = client.get_oembed("https://example.com/abc123").await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid url: URL must start with 'https://gyazo.com/'"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_params_builder() -> anyhow::Result<()> {
        let params = UploadParamsBuilder::new(vec![0, 1, 2, 3])
            .access_policy("anyone")?
            .metadata_is_public("true")?
            .referer_url("https://example.com")
            .app("test app")
            .title("test image")
            .desc("test description")
            .created_at("2024-08-10 12:00:00")
            .collection_id("test collection")
            .build()?;

        assert_eq!(params.imagedata, vec![0, 1, 2, 3]);
        assert_eq!(params.access_policy, Some("anyone".to_string()));
        assert_eq!(params.metadata_is_public, Some("true".to_string()));
        assert_eq!(params.referer_url, Some("https://example.com".to_string()));
        assert_eq!(params.app, Some("test app".to_string()));
        assert_eq!(params.title, Some("test image".to_string()));
        assert_eq!(params.desc, Some("test description".to_string()));
        assert_eq!(params.created_at, Some("2024-08-10 12:00:00".to_string()));
        assert_eq!(params.collection_id, Some("test collection".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_params_builder_invalid_access_policy() -> anyhow::Result<()> {
        let result = UploadParamsBuilder::new(vec![0, 1, 2, 3])
            .access_policy("invalid")
            .unwrap_err();
        assert_eq!(
            result.to_string(),
            "Invalid input: access_policy must be 'anyone' or 'only_me'"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_params_builder_invalid_metadata_is_public() -> anyhow::Result<()> {
        let result = UploadParamsBuilder::new(vec![0, 1, 2, 3])
            .metadata_is_public("invalid")
            .unwrap_err();
        assert_eq!(
            result.to_string(),
            "Invalid input: metadata_is_public must be 'true' or 'false'"
        );
        Ok(())
    }
}
