# Gyazo Client for Rust

A Rust library for interacting with the Gyazo API. Upload, retrieve, list, and delete images on Gyazo efficiently.

[![ci](https://github.com/katayama8000/gyazo-client-rust/workflows/ci/badge.svg)](https://github.com/katayama8000/gyazo-client-rust/actions)
[![crates.io](https://img.shields.io/crates/v/gyazo_client)](https://crates.io/crates/gyazo_client)
[![docs.rs](https://img.shields.io/docsrs/gyazo_client)](https://docs.rs/gyazo_client)
[![license](https://img.shields.io/crates/l/gyazo_client)](LICENSE)

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
    let gyazo_client = GyazoClient::new("YOUR_ACCESS_TOKEN".to_string());

    // Upload an image with a title and metadata_is_public
    let image_data = std::fs::read("path/to/your/image.png")?;
    let upload_params = UploadParamsBuilder::new(image_data)
        .title("My awesome image".to_string())
        .metadata_is_public("true".to_string())
        .build()?;
    let upload_response = gyazo_client.upload_image(upload_params).await?;

    // Get image
    let image = gyazo_client.get_image(&upload_response.image_id).await?;

    // List images
    let images = gyazo_client.list_images().await?;

    // Delete an image
    let delete_response = gyazo_client.delete_image(&upload_response.image_id).await?;

    // get oEmbed URL
    let oembed_url = gyazo_client.get_oembed_url(&upload_response.image_id);

    Ok(())
}
```

## References
- [Gyazo API Documentation](https://gyazo.com/api/docs/image)