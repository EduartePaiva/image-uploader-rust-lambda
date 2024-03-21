use aws_sdk_s3::Client as S3Client;
use image::{codecs::jpeg::JpegEncoder, GenericImageView};
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use s3::{GetFile, PutFile};
use serde::{Deserialize, Serialize};
use std::env;

mod s3;

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
async fn function_handler<T: PutFile + GetFile>(
    event: LambdaEvent<Request>,
    client: &T,
    bkt_names: &BucketNames,
) -> Result<Response, Error> {
    // Extract some useful info from the request

    let (image_to_process, metadata) = client
        .get_file(&bkt_names.bucket_to_get_image, &event.payload.image_name)
        .await?;
    let mut image_to_process = image::load_from_memory(&image_to_process)?;

    let (width, height) = image_to_process.dimensions();

    if metadata.portrait_hight + metadata.y1 > height
        || metadata.portrait_width + metadata.x1 > width
    {
        //instead of erroing I could just resize it and call a day
        return Err("Invalid metadata".into());
    }

    let mut writer: Vec<u8> = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut writer, 72);

    image_to_process
        .crop(
            metadata.x1,
            metadata.y1,
            metadata.portrait_width,
            metadata.portrait_hight,
        )
        .write_with_encoder(encoder)?;

    client
        .put_file(
            &bkt_names.bucket_to_put_image,
            &event.payload.image_name,
            writer,
        )
        .await?;

    // Prepare the response
    let resp = Response { success: true };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
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
