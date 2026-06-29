#!/usr/bin/env bash
# 开发启动脚本：自动检测 Wayland 会话并设置 WebKit 环境变量，
# 解决部分 Wayland 桌面下 WebView 白屏/渲染异常的问题。
set -e

if [ -n "$WAYLAND_DISPLAY" ] || [ "${XDG_SESSION_TYPE:-}" = "wayland" ]; then
  export WEBKIT_DISABLE_COMPOSITING_MODE=1
  echo "[run-dev] Wayland detected, set WEBKIT_DISABLE_COMPOSITING_MODE=1"
fi

cargo tauri dev "$@"
