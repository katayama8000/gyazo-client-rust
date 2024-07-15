# Gyazo Client for Rust

A Rust library for interacting with the Gyazo API. Upload, retrieve, list, and delete images on Gyazo efficiently.

## Features

- Upload, retrieve, list, and delete images
- Asynchronous API using tokio and reqwest
- Custom error handling

## Installation

```sh
cargo add gyazo_client
```

## Usage

```rust
use gyazo_client::{GyazoClient, UploadParams, UploadParamsBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Gyazo client with your access token
    let client = GyazoClient::new("YOUR_ACCESS_TOKEN".to_string());

    // Upload an image with a title
    let image_data = std::fs::read("path/to/your/image.png")?;
    let upload_params = UploadParamsBuilder::new(image_data)
        .title("My awesome image".to_string())
        .metadata_is_public("true".to_string())
        .build()?;
    let upload_response = client.upload_image(upload_params).await?;

    // Get image information
    let image_info = client.get_image(&upload_response.image_id).await?;

    // List images
    let images = client.list_images().await?;

    // Delete an image
    let delete_response = client.delete_image(&upload_response.image_id).await?;

    Ok(())
}
```

## References
- [Gyazo API Documentation](https://gyazo.com/api/docs/image)