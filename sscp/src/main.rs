mod s3;
mod transfer;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_ssm::Client as SsmClient;
use clap::Parser;
use s3::S3Url;
use slog::{o, Drain};
use std::io::{self};
use transfer::{LocalUsingS3Hub, S3Hub, SsmEc2UsingS3Hub};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    src_path: String,
    dest_path: String,
    #[arg(long)]
    bucket: String,
}

fn logger() -> slog::Logger {
    let drain = slog_json::Json::new(io::stdout())
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

async fn make_hub(path: impl ToString) -> Result<Box<dyn S3Hub>, String> {
    let p = path.to_string();
    let s: Vec<&str> = p.split(':').collect();
    let region_provider = RegionProviderChain::default_provider();
    let config = aws_config::from_env().region(region_provider).load().await;

    match s[..] {
        [p] => {
            // S3にファイルをアップロード
            let client = S3Client::new(&config);
            Ok(Box::new(LocalUsingS3Hub::new(client, p.to_string())))
        }
        [instance_id, p] => {
            let client = SsmClient::new(&config);

            Ok(Box::new(SsmEc2UsingS3Hub::new(
                client,
                p.to_string(),
                instance_id.to_string(),
            )))
        }
        _ => Err(format!("{} is not expected", p)),
    }
}

#[tokio::main]
async fn main() {
    let logger = logger();
    let args = Args::parse();

    let src_path = args.src_path;
    let dest_path = args.dest_path;

    slog::debug!(
        logger,
        "cp {src_path} {dest_path}",
        src_path = &src_path,
        dest_path = &dest_path
    );

    let s3_url = S3Url::new(args.bucket, &src_path);

    let src = make_hub(&src_path).await.unwrap();
    let dest = make_hub(&dest_path).await.unwrap();

    src.transfer_to(dest, &s3_url).await.unwrap();
}
