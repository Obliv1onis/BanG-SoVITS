# BanG Voice Studio

面向 macOS 的《BanG Dream!》角色语音生成桌面应用。它使用本地 GPT-SoVITS v2ProPlus 权重和角色语音素材，素材与生成结果不会上传到云端。

## 功能

- 按乐队和角色浏览本地声线
- 自动以音频文件名作为日文参考台词
- 自动配对角色的 `.ckpt` / `.pth` 权重
- 中文、日文台词生成，支持语速与随机度调节
- 原生 macOS 窗口、音频试听与导出

## macOS 开发运行

要求：Apple Silicon Mac、Node.js 20+、Rust、uv、ffmpeg。

```bash
chmod +x scripts/*.sh
./scripts/setup-macos.sh
npm install
npm run tauri:dev
```

`setup-macos.sh` 会固定检出经过本项目验证的 GPT-SoVITS 版本并应用 macOS 补丁。模型和角色语音不包含在 Git 仓库中，请自行准备以下目录：

```text
models/GPT_weights_v2ProPlus/
models/SoVITS_weights_v2ProPlus/
models/pretrained_models/
角色语音/<乐队>/<角色>/
```

构建可安装应用：

```bash
npm run tauri:build
```

产物位于 `src-tauri/target/release/bundle/`。

## 目录说明

- `src/`：专用桌面界面
- `src-tauri/`：macOS 原生窗口、素材扫描、推理引擎生命周期与文件导出
- `engine/`：来自官方 GitHub 的最新 GPT-SoVITS macOS 推理内核与项目虚拟环境
- `models/`：52 对角色权重和 v2ProPlus 基础模型；新引擎通过符号链接读取
- `角色语音/`：参考语音；文件名就是参考台词

## 本地推理接口

应用在需要生成时启动原项目的 `api_v2.py`，仅监听 `127.0.0.1:9880`。兼容接口示例：

```bash
curl -X POST http://127.0.0.1:9880/tts \
  -H 'Content-Type: application/json' \
  -d '{"text":"こんにちは","text_lang":"ja","ref_audio_path":"/absolute/reference.mp3","prompt_text":"参考台词","prompt_lang":"ja"}' \
  --output voice.wav
```

Python 与 JavaScript 客户端同样向 `/tts` 发送上述 JSON；切换角色时分别调用 `/set_gpt_weights` 与 `/set_sovits_weights`。完整字段见原项目 `api_v2.py` 顶部文档。

## 使用声明

本项目是非官方本地工具。请仅在拥有素材与模型使用权的范围内使用；不要用于冒充真人、欺骗或未经授权的商业发布。原 GPT-SoVITS 的 `LICENSE` 保留在引擎目录中。

## GitHub 发布说明

完整离线版约 22 GB，DMG 约 19 GB，均超过 GitHub 的单文件托管限制，因此仓库只保存应用源码、配置和环境脚本，不保存模型、参考语音、Python 虚拟环境或构建产物。
