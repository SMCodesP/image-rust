mod image_processor;

use std::time::Instant;

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use base64::{prelude::BASE64_STANDARD, Engine};
use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};
use percent_encoding::percent_decode;
use serde_json::{json, Value};

const TRANSFORMED_IMAGE_CACHE_TTL: &str = "max-age=3600";
const S3_BUCKET_ORIGINAL: &str = "mw-cms-media";
const S3_BUCKET_OPTIMIZED: &str = "mw-cms-optimized";
const ORIGINAL_OPERATION: &str = "original";

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    lambda_runtime::run(service_fn(handler)).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    let (event, _) = event.into_parts();
    let path = event["rawPath"].as_str().unwrap_or("/");

    let (operations, original_path) = extract_path_components(path);

    let start_client = Instant::now();
    let s3_client = create_s3_client().await;
    println!(
        "Time to create S3 client: {} ms",
        start_client.elapsed().as_millis()
    );

    let start_download = Instant::now();
    let (image_data, content_type) = download_original_image(&s3_client, &original_path).await?;
    println!(
        "Time to download image: {} ms",
        start_download.elapsed().as_millis()
    );

    let should_transform =
        is_transformable_content_type(&content_type) && operations != ORIGINAL_OPERATION;
    let (processed_image, cache_operation) = if should_transform {
        match image_processor::process_image(&image_data, &content_type, operations).await {
            Ok(image) => (image, operations.to_string()),
            Err(error) => {
                eprintln!(
                    "Image transform failed for '{original_path}' with operations '{operations}': {error}. Returning original file."
                );
                (image_data.clone(), ORIGINAL_OPERATION.to_string())
            }
        }
    } else {
        println!(
            "Bypassing transformation for content-type '{}' and path '{}'",
            content_type, original_path
        );
        (image_data.clone(), ORIGINAL_OPERATION.to_string())
    };

    let bg_client = s3_client.clone();
    let bg_path = original_path.clone();
    let bg_ops = cache_operation;
    let bg_image = processed_image.clone();
    let bg_content_type = content_type.clone();

    let start_background = Instant::now();
    tokio::spawn(async move {
        if let Err(e) =
            background_processing(bg_client, bg_path, bg_ops, bg_image, bg_content_type).await
        {
            eprintln!("Background processing failed: {:?}", e);
        }
    });
    println!(
        "Time to start background processing: {} ms",
        start_background.elapsed().as_millis()
    );

    Ok(build_response(200, &content_type, &processed_image))
}

async fn background_processing(
    client: S3Client,
    original_path: String,
    operations: String,
    image_data: Vec<u8>,
    content_type: String,
) -> Result<(), LambdaError> {
    let target_path = format!("{}/{}", original_path, operations);

    client
        .put_object()
        .bucket(S3_BUCKET_OPTIMIZED)
        .key(target_path)
        .content_type(&content_type)
        .body(image_data.into())
        .send()
        .await?;

    Ok(())
}

fn decode_url_path(path: &str) -> String {
    return percent_decode(path.as_bytes())
        .decode_utf8_lossy()
        .to_string();
}

fn extract_path_components(path: &str) -> (&str, String) {
    let mut parts: Vec<_> = path.split('/').collect();
    let operations = parts.pop().unwrap_or("");
    let original_path_encoded = parts[1..].join("/");
    let original_path = decode_url_path(&original_path_encoded);
    (operations, original_path)
}

async fn create_s3_client() -> S3Client {
    let shared_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    S3Client::new(&shared_config)
}

async fn download_original_image(
    client: &S3Client,
    path: &str,
) -> Result<(Vec<u8>, String), LambdaError> {
    let response = client
        .get_object()
        .bucket(S3_BUCKET_ORIGINAL)
        .key(path)
        .send()
        .await?;

    let content_type = response
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
    let data = response.body.collect().await?.to_vec();
    Ok((data, content_type))
}

fn build_response(status: u16, content_type: &str, body: &[u8]) -> Value {
    json!({
        "statusCode": status,
        "headers": {
            "Content-Type": content_type,
            "Cache-Control": TRANSFORMED_IMAGE_CACHE_TTL,
        },
        "body": BASE64_STANDARD.encode(body),
        "isBase64Encoded": true
    })
}

fn is_transformable_content_type(content_type: &str) -> bool {
    let normalized = content_type.to_ascii_lowercase();

    normalized.starts_with("image/jpeg")
        || normalized.starts_with("image/jpg")
        || normalized.starts_with("image/png")
        || normalized.starts_with("image/webp")
        || normalized.starts_with("image/avif")
}
