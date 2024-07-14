Copy file to an EC2 instance that can only be accessed via SSM and not using SSH through S3.

```sh
# local to ec2
sscp 'Cargo.toml' 'i-0aa0123456789abcd:~/Cargo2.toml' --bucket bucket-name
```
