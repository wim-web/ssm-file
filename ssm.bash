#!/bin/bash

# インスタンスのリストを取得し、pecoで選択
INSTANCE_ID=$(aws ec2 describe-instances \
  --query "Reservations[*].Instances[*].[InstanceId, Tags[?Key=='Name'].Value|[0], State.Name]" \
  --output text | \
  peco | \
  awk '{print $1}')

# 選択されたインスタンスIDを表示
echo "Selected Instance ID: $INSTANCE_ID"

# SSMセッションを開始
if [ -n "$INSTANCE_ID" ]; then
  aws ssm start-session --target "$INSTANCE_ID"
else
  echo "No instance selected. Exiting."
fi
