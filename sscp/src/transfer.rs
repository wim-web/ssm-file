use std::time::Duration;

use aws_sdk_s3::primitives::ByteStream;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    time::sleep,
};

use crate::{s3::S3Url, S3Client, SsmClient};

#[async_trait::async_trait]
#[allow(warnings)]
pub(crate) trait S3Hub: internal::S3HubInternal {
    async fn transfer_to(
        &self,
        dest: Box<dyn S3Hub>,
        url: &S3Url,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.upload_to_s3(url).await?;
        dest.download_from_s3(url).await?;

        Ok(())
    }
}

mod internal {
    use std::error::Error;

    use crate::s3::S3Url;

    #[async_trait::async_trait]
    pub(super) trait S3HubInternal: Send + Sync {
        async fn upload_to_s3(&self, url: &S3Url) -> Result<(), Box<dyn Error>>;
        async fn download_from_s3(&self, url: &S3Url) -> Result<(), Box<dyn Error>>;
    }
}

pub struct LocalUsingS3Hub {
    client: S3Client,
    path: String,
}

impl LocalUsingS3Hub {
    pub fn new(client: S3Client, path: String) -> Self {
        LocalUsingS3Hub { client, path }
    }
}

#[async_trait::async_trait]
impl internal::S3HubInternal for LocalUsingS3Hub {
    async fn upload_to_s3(&self, url: &S3Url) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(&self.path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let byte_stream = ByteStream::from(buffer);

        self.client
            .put_object()
            .bucket(&url.bucket)
            .key(&url.key)
            .body(byte_stream)
            .send()
            .await?;

        Ok(())
    }

    async fn download_from_s3(&self, url: &S3Url) -> Result<(), Box<dyn std::error::Error>> {
        let content = self
            .client
            .get_object()
            .bucket(&url.bucket)
            .key(&url.key)
            .send()
            .await?;

        let body = content.body.collect().await?;

        let mut file = File::create(&self.path).await?;
        file.write_all(&body.into_bytes()).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl S3Hub for LocalUsingS3Hub {}

pub struct SsmEc2UsingS3Hub {
    client: SsmClient,
    path: String,
    instance_id: String,
}

impl SsmEc2UsingS3Hub {
    pub fn new(client: SsmClient, path: String, instance_id: String) -> Self {
        SsmEc2UsingS3Hub {
            client,
            path,
            instance_id,
        }
    }

    async fn ssm_send_command(
        &self,
        commands: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let send_command_result = self
            .client
            .send_command()
            .instance_ids(&self.instance_id)
            .document_name("AWS-RunShellScript")
            .parameters("commands", commands)
            .send()
            .await?;

        let command_id = send_command_result.command().unwrap().command_id().unwrap();

        // コマンドの結果をポーリングして取得
        loop {
            let command_invocation = self
                .client
                .get_command_invocation()
                .command_id(command_id)
                .instance_id(&self.instance_id)
                .send()
                .await?;

            match command_invocation.status().unwrap().as_str() {
                "InProgress" | "Pending" => {
                    sleep(Duration::from_secs(2)).await;
                }
                "Success" => {
                    println!(
                        "Command executed successfully: {}",
                        command_invocation.standard_output_content().unwrap()
                    );
                    return Ok(());
                }
                "Failed" | "TimedOut" | "Cancelled" | "Cancelling" => {
                    return Err(format!(
                        "Command execution failed: {}",
                        command_invocation.standard_error_content().unwrap()
                    )
                    .into());
                }
                _ => {
                    eprintln!(
                        "Unknown command status: {}",
                        command_invocation.status().unwrap().as_str()
                    );
                    return Err(format!(
                        "Unknown command status: {}",
                        command_invocation.status().unwrap().as_str()
                    )
                    .into());
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl internal::S3HubInternal for SsmEc2UsingS3Hub {
    async fn upload_to_s3(&self, url: &S3Url) -> Result<(), Box<dyn std::error::Error>> {
        let command = format!("aws s3 cp {} {}", self.path, url.url());
        self.ssm_send_command(vec![command]).await
    }

    async fn download_from_s3(&self, url: &S3Url) -> Result<(), Box<dyn std::error::Error>> {
        let command = format!("aws s3 cp {} {}", url.url(), self.path);
        self.ssm_send_command(vec![command]).await
    }
}

#[async_trait::async_trait]
impl S3Hub for SsmEc2UsingS3Hub {}
