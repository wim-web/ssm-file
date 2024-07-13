# S3バケット (パブリックアクセス禁止)
resource "aws_s3_bucket" "example" {
  bucket = "0xb02efd083189edae326587e448755249dbdd3b2a7b9c47f41e"
}

resource "aws_s3_bucket_server_side_encryption_configuration" "example_sse" {
  bucket = aws_s3_bucket.example.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}
