# Terraform設定ブロック
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.57.0"
    }
  }

  required_version = "1.9.2"
}

# AWSプロバイダー設定
provider "aws" {
  region = "us-west-2"
}








