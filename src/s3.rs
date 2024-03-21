use async_trait::async_trait;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::tracing;

pub struct ImageMetadata {
    pub content_type: String,
    pub portrait_hight: u32,
    pub portrait_width: u32,
    pub x1: u32,
    pub y1: u32,
}

#[async_trait]
pub trait GetFile {
    async fn get_file(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Vec<u8>, ImageMetadata), GetObjectError>;
}

#[async_trait]
pub trait PutFile {
    async fn put_file(&self, bucket: &str, key: &str, bytes: Vec<u8>) -> Result<String, String>;
}

#[async_trait]
impl GetFile for S3Client {
    async fn get_file(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Vec<u8>, ImageMetadata), GetObjectError> {
        tracing::info!("get file bucket {}, key {}", bucket, key);

        let output = self.get_object().bucket(bucket).key(key).send().await;

        return match output {
            Ok(response) => {
                let metadata = response.metadata.expect("Image don't have metadata");
                let content_type = metadata
                    .get("Content-Type".into())
                    .expect("should have content type")
                    .clone();
                let portrait_hight: u32 = metadata
                    .get("x-amz-meta-portraithight".into())
                    .expect("should have portrait hight")
                    .parse()
                    .expect("Should have parsed");
                let portrait_width: u32 = metadata
                    .get("x-amz-meta-portraitwidth".into())
                    .expect("should have portrait width")
                    .parse()
                    .expect("Should have parsed");
                let x1: u32 = metadata
                    .get("x-amz-meta-x1".into())
                    .expect("should have x1")
                    .parse()
                    .expect("Should have parsed");
                let y1: u32 = metadata
                    .get("x-amz-meta-y1".into())
                    .expect("should have y1")
                    .parse()
                    .expect("Should have parsed");
                let metadata = ImageMetadata {
                    content_type,
                    portrait_hight,
                    portrait_width,
                    x1,
                    y1,
                };

                let bytes = response.body.collect().await.unwrap().to_vec();
                tracing::info!("Object is downloaded, size is {}", bytes.len());
                Ok((bytes, metadata))
            }
            Err(err) => {
                let service_err = err.into_service_error();
                let meta = service_err.meta();
                tracing::info!("Error from aws when downloding: {}", meta.to_string());
                Err(service_err)
            }
        };
    }
}

#[async_trait]
impl PutFile for S3Client {
    async fn put_file(&self, bucket: &str, key: &str, vec: Vec<u8>) -> Result<String, String> {
        tracing::info!("put file bucket {}, key {}", bucket, key);
        let bytes = ByteStream::from(vec);
        let result = self
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(bytes)
            .send()
            .await;

        match result {
            Ok(_) => Ok(format!("Uploaded a file with key {} into {}", key, bucket)),
            Err(err) => Err(err
                .into_service_error()
                .meta()
                .message()
                .unwrap()
                .to_string()),
        }
    }
}
