use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{config::{ProvideCredentials, SharedCredentialsProvider}, Client as S3Client};
use base64::{engine::general_purpose, Engine};
use image::{EncodableLayout, ImageError, ImageFormat};
use lambda_runtime::{service_fn, LambdaEvent, Error as LambdaError};
use serde_json::{json, Value};
use std::io::Cursor;
use webp::{Encoder as WebPEncoder, PixelLayout};

// const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB, ajuste conforme necessário
const TRANSFORMED_IMAGE_CACHE_TTL: &str = "max-age=3600";

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    let start_all = std::time::Instant::now();
    let (event, _context) = event.into_parts();
    
    let path = event["rawPath"].as_str().unwrap_or("/");
    let mut path_parts: Vec<&str> = path.split('/').collect();
    let operations_prefix = path_parts.pop().unwrap_or("");
    let original_image_path = path_parts[1..].join("/");
    
    let region = std::env::var("AWS_REGION").unwrap();

    let mut start = std::time::Instant::now();
    let conf =  aws_config::SdkConfig::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(region))
        .credentials_provider(SharedCredentialsProvider::new(StaticCredentials::new()))
        .build();
    eprintln!("Time of load arstarst: {:?}", start.elapsed());
    start = std::time::Instant::now();
    let s3_client = S3Client::new(&conf);
    eprintln!("Time of create client: {:?}", start.elapsed());

    start = std::time::Instant::now();
    let original_image = match download_original_image(&s3_client, &original_image_path).await {
        Ok((body, content_type)) => (body, content_type),
        // Err(err) => return send_error(500, "Error downloading original image ", &err.to_string()),
        Err(err) => return send_error(500, "Error downloading original image", &err.to_string()),
    };
    eprintln!("Time of download original image: {:?}", start.elapsed());

    start = std::time::Instant::now();
    let transformed_image = match process_image(&original_image.0, operations_prefix).await {
        Ok(img) => img,
        Err(err) => return send_error(500, "Error processing image", &err.to_string()),
    };
    eprintln!("Time of process image: {:?}", start.elapsed());

    eprintln!("Time of all process: {:?}", start_all.elapsed());
    
    Ok(json!({
        "statusCode": 200,
        "headers": {
            "Content-Type": original_image.1,
            "Cache-Control": TRANSFORMED_IMAGE_CACHE_TTL,
        },
        "body": general_purpose::STANDARD.encode(&transformed_image),
        "isBase64Encoded": true
    }))
}

async fn download_original_image(
    s3_client: &S3Client,
    path: &str,
) -> Result<(Vec<u8>, String), LambdaError> {
    let bucket_name = "img-resizo";

    let output = s3_client
    .get_object()
    .bucket(bucket_name)
    .key(path)
    .send()
    .await
        .map_err(|err| {
            eprintln!("Erro: {:?}", err);
            LambdaError::from(err)
        })?;

    let content_type = output.content_type().unwrap_or("application/octet-stream").to_string();
    
    let body = output.body.collect().await.map_err(|err| LambdaError::from(err))?; // Tratando erro da coleta dos bytes

    Ok((body.to_vec(), content_type))
}

async fn process_image(
    image_data: &[u8],
    operations: &str,
) -> Result<Vec<u8>, ImageError> {
    let mut img = image::load_from_memory(image_data)?;

    // Parse operations (mantido igual)
    let operations_map: std::collections::HashMap<_, _> = operations
        .split(',')
        .filter_map(|op| {
            let mut parts = op.split('=');
            Some((parts.next()?, parts.next()?))
        })
        .collect();

    if let Some(width) = operations_map.get("width").and_then(|w| w.parse::<u32>().ok()) {
        img = img.resize(width, img.height(), image::imageops::FilterType::Triangle);
    }

    // Determinar formato e content_type
    let (format, _content_type) = match operations_map.get("format") {
        Some(&"png") => (ImageFormat::Png, "image/png"),
        Some(&"webp") => (ImageFormat::WebP, "image/webp"),
        _ => (ImageFormat::Jpeg, "image/jpeg"),
    };

    let mut buf = Vec::new();

    // Codificação específica para WebP
    if let ImageFormat::WebP = format {
        let rgba = img.to_rgba8();
        let encoder = WebPEncoder::new(&rgba, PixelLayout::Rgba, img.width(), img.height());
        let quality = 75.0;
        let webp_data = encoder.encode(quality);
        buf = webp_data.as_bytes().to_vec();
    } else {
        img.write_to(&mut Cursor::new(&mut buf), format)?;
    }

    Ok(buf)
}

fn send_error(status_code: u16, message: &str, error: &str) -> Result<Value, LambdaError> {
    eprintln!("Error: {} - {}", message, error);
    Ok(json!({
        "statusCode": status_code,
        "body": message,
    }))
}


#[derive(Debug)]
struct StaticCredentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
}

impl StaticCredentials {
    pub fn new() -> Self {
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID").unwrap();
        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY").unwrap();
        let aws_session_token = std::env::var("AWS_SESSION_TOKEN").unwrap_or_default();
        Self {
            access_key_id: access_key_id.trim().to_string(),
            secret_access_key: secret_access_key.trim().to_string(),
            session_token: aws_session_token.trim().to_string(),
        }
    }

    async fn load_credentials(&self) -> aws_credential_types::provider::Result {
        Ok(Credentials::new(
            self.access_key_id.clone(),
            self.secret_access_key.clone(),
            Some(self.session_token.clone()),
            None,
            "StaticCredentials",
        ))
    }
}

impl ProvideCredentials for StaticCredentials {
    fn provide_credentials<'a>(
        &'a self,
    ) -> aws_credential_types::provider::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        aws_credential_types::provider::future::ProvideCredentials::new(self.load_credentials())
    }
}