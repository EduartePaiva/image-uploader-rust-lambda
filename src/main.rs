use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use image::{codecs::jpeg::JpegEncoder, GenericImageView};
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use rust_lambda_image_uploader::ImageMetadata;
use serde::{Deserialize, Serialize};
use std::env;

const MAX_IMAGE_SIZE: u32 = 400;

/// This is a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    image_name: String,
}

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
struct Response {
    success: bool,
}

struct BucketNames {
    bucket_to_get_image: String,
    bucket_to_put_image: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(
    event: LambdaEvent<Request>,
    client: &S3Client,
    bkt_names: &BucketNames,
) -> Result<Response, Error> {
    let key_name = &event.payload.image_name;

    //get image from S3
    let output = client
        .get_object()
        .bucket(&bkt_names.bucket_to_get_image)
        .key(key_name)
        .send()
        .await?;
    let metadata = ImageMetadata::new(&output.metadata.ok_or("Should have metadata")?)?;

    //check if image is png or jpeg or webp or jpg
    let content_type = output.content_type.ok_or("Should have content type")?;
    match content_type.as_str() {
        "image/jpg" => (),
        "image/jpeg" => (),
        "image/png" => (),
        "image/webp" => (),
        _ => return Err("Invalid file type".into()),
    }
    // -----------------------

    let mut image_to_process = image::load_from_memory(&output.body.collect().await?.to_vec())?;
    let (width, height) = image_to_process.dimensions();
    // ------------------------

    // Crop, resize and convert the  image to jpeg with quality of 75dpi
    let mut resultant_image: Vec<u8> = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut resultant_image, 75);

    if metadata.portrait_hight + metadata.y1 > height
        || metadata.portrait_width + metadata.x1 > width
    {
        image_to_process.thumbnail(MAX_IMAGE_SIZE, MAX_IMAGE_SIZE)
    } else {
        image_to_process
            .crop(
                metadata.x1,
                metadata.y1,
                metadata.portrait_width,
                metadata.portrait_hight,
            )
            .thumbnail(MAX_IMAGE_SIZE, MAX_IMAGE_SIZE)
    }
    .into_rgb8()
    .write_with_encoder(encoder)?;
    // ---------------------------

    // Send image to the other bucket using the "resultant_image"
    let bucket_to_put_image = &bkt_names.bucket_to_put_image;
    let bytes = ByteStream::from(resultant_image);
    let result = client
        .put_object()
        .bucket(bucket_to_put_image)
        .key(key_name)
        .body(bytes)
        .content_type("image/jpeg")
        .send()
        .await;

    match result {
        Ok(_) => (),
        Err(err) => return Err(err.into_service_error().meta().message().unwrap().into()),
    };
    // ---------------------------

    // delete image from primary bucket
    let result = client
        .delete_object()
        .bucket(&bkt_names.bucket_to_get_image)
        .key(key_name)
        .send()
        .await;

    match result {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("failed to delete original image {:?}", err);
        }
    }
    //-------------------

    let resp = Response { success: true };
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let shared_config = aws_config::load_from_env().await;
    let client = S3Client::new(&shared_config);
    let client_ref = &client;

    let bkt_names = BucketNames {
        bucket_to_get_image: env::var("BUCKET_TO_GET_IMAGE")?,
        bucket_to_put_image: env::var("BUCKET_TO_PUT_IMAGE")?,
    };
    let bkt_name_ref = &bkt_names;

    let func =
        service_fn(
            move |event| async move { function_handler(event, client_ref, bkt_name_ref).await },
        );

    run(func).await?;

    Ok(())
}
