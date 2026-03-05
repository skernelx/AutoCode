#!/bin/bash

# 生成 Tauri 更新所需的 latest.json 文件
# 使用方法: ./generate-update-json.sh <version> <notes>

VERSION=$1
NOTES=$2

if [ -z "$VERSION" ]; then
  echo "用法: $0 <version> <notes>"
  echo "示例: $0 0.2.2 '修复bug'"
  exit 1
fi

# 获取 DMG 文件路径
DMG_FILE="src-tauri/target/release/bundle/dmg/AutoCode_${VERSION}_aarch64.dmg"

if [ ! -f "$DMG_FILE" ]; then
  echo "错误: 找不到 DMG 文件: $DMG_FILE"
  exit 1
fi

# 计算签名（这里使用 SHA256，实际应该使用私钥签名）
SIGNATURE=$(shasum -a 256 "$DMG_FILE" | awk '{print $1}')

# 获取文件大小
SIZE=$(stat -f%z "$DMG_FILE")

# 生成 JSON
cat > latest.json <<EOF
{
  "version": "${VERSION}",
  "notes": "${NOTES}",
  "pub_date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "platforms": {
    "darwin-aarch64": {
      "signature": "${SIGNATURE}",
      "url": "https://github.com/skernelx/AutoCode/releases/download/v${VERSION}/AutoCode_${VERSION}_aarch64.dmg"
    }
  }
}
EOF

echo "✅ 已生成 latest.json"
cat latest.json
