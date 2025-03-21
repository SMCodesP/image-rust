mod image_processor;

use std::time::Instant;

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use base64::{prelude::BASE64_STANDARD, Engine};
use lambda_runtime::{service_fn, LambdaEvent, Error as LambdaError};
use serde_json::{json, Value};

const TRANSFORMED_IMAGE_CACHE_TTL: &str = "max-age=3600";

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
    println!("Tempo para criar cliente S3: {} ms", start_client.elapsed().as_millis());
    
    let start_download = Instant::now();
    let (image_data, content_type) = download_original_image(&s3_client, &original_path).await?;
    println!("Tempo para baixar imagem: {} ms", start_download.elapsed().as_millis());
    let processed_image = image_processor::process_image(&image_data, &content_type, operations).await?;

    let bg_client = s3_client.clone();
    let bg_path = original_path.clone();
    let bg_ops = operations.to_string();
    let bg_image = processed_image.clone();
    let bg_content_type = content_type.clone();

    let start_background = Instant::now();
    tokio::spawn(async move {
        if let Err(e) = background_processing(
            bg_client,
            bg_path,
            bg_ops,
            bg_image,
            bg_content_type
        ).await {
            eprintln!("Background processing failed: {:?}", e);
        }
    });
    println!("Tempo para iniciar processamento em background: {} ms", start_background.elapsed().as_millis());

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
        .bucket("comprautos-static-optimized")
        .key(target_path)
        .content_type(&content_type)
        .body(image_data.into())
        .send()
        .await?;

    Ok(())
}

// Componentes refatorados
fn extract_path_components(path: &str) -> (&str, String) {
    let mut parts: Vec<_> = path.split('/').collect();
    let operations = parts.pop().unwrap_or("");
    let original_path = parts[1..].join("/");
    (operations, original_path)
}

async fn create_s3_client() -> S3Client {
    let shared_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    S3Client::new(&shared_config)
}

async fn download_original_image(client: &S3Client, path: &str) -> Result<(Vec<u8>, String), LambdaError> {
    let response = client
        .get_object()
        .bucket("comprautos-static")
        .key(path)
        .send()
        .await?;

    let content_type = response.content_type().unwrap_or("application/octet-stream").to_string();
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