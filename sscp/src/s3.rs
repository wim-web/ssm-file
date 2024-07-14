pub struct S3Url {
    pub bucket: String,
    pub key: String,
}

impl S3Url {
    pub fn new(bucket: impl ToString, key: impl ToString) -> Self {
        S3Url {
            bucket: bucket.to_string(),
            key: key.to_string(),
        }
    }

    pub fn url(&self) -> String {
        format!("s3://{}/{}", self.bucket, self.key)
    }
}
