resource "aws_s3_bucket" "example" {
  bucket        = var.bucket_name
  force_destroy = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "example_sse" {
  bucket = aws_s3_bucket.example.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}
