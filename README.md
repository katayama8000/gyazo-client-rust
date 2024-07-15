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
use gyazo_client::{GyazoClient, UploadParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GyazoClient::new("YOUR_ACCESS_TOKEN".to_string());

    // Upload an image
    let image_data = std::fs::read("path/to/your/image.png")?;
    let upload_params = UploadParams {
        imagedata: image_data,
        title: Some("My awesome image".to_string()),
        ..Default::default()
    };
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