#!/bin/zsh
set -e
ROOT="${0:A:h:h}"
ENGINE="$ROOT/engine"

if [[ ! -d "$ENGINE/.git" ]]; then
  git clone https://github.com/RVC-Boss/GPT-SoVITS.git "$ENGINE"
  git -C "$ENGINE" checkout be6a4f1e9d8a22d41b7d42c22df9d7ef36f225d2
  git -C "$ENGINE" apply "$ROOT/patches/gpt-sovits-macos.patch"
fi

UV="$(command -v uv 2>/dev/null || true)"
[[ -z "$UV" && -x "$HOME/.local/bin/uv" ]] && UV="$HOME/.local/bin/uv"
if [[ -z "$UV" ]]; then
  echo "未找到 uv。请先安装：curl -LsSf https://astral.sh/uv/install.sh | sh"
  exit 1
fi

cd "$ENGINE"
"$UV" python install 3.11
"$UV" venv --python 3.11 .venv
"$UV" pip install --python .venv/bin/python torch torchvision torchaudio torchcodec
"$UV" pip install --python .venv/bin/python -r requirements.txt

if [[ -d "$ROOT/models/GPT_weights_v2ProPlus" ]]; then
  ln -sfn ../models/GPT_weights_v2ProPlus GPT_weights_v2ProPlus
fi
if [[ -d "$ROOT/models/SoVITS_weights_v2ProPlus" ]]; then
  ln -sfn ../models/SoVITS_weights_v2ProPlus SoVITS_weights_v2ProPlus
fi
if [[ -d "$ROOT/models/pretrained_models" ]]; then
  rm -rf GPT_SoVITS/pretrained_models
  ln -s ../../models/pretrained_models GPT_SoVITS/pretrained_models
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "未找到 ffmpeg。请运行：brew install ffmpeg"
fi

echo "macOS 推理环境安装完成。"
