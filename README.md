# BanG Voice Studio

面向 macOS 的《BanG Dream!》角色语音生成桌面应用，基于 GPT-SoVITS。本地完成模型加载与语音生成，输入台词和生成结果不会上传到云端。

## 系统要求

- Apple Silicon Mac（M1、M2、M3、M4）
- macOS 12 或更高版本
- 建议至少 16 GB 内存
- 安装后约占用 22 GB 磁盘空间

## 下载完整应用

请前往 [Releases](https://github.com/Obliv1onis/BanG-SoVITS/releases) 下载最新版本。

由于完整 DMG 超过 GitHub 的单文件大小限制，安装包被分成 11 个文件。请下载 `part-00` 至 `part-10` 的全部分卷，缺少任何一个都无法安装：

```text
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-00
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-01
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-02
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-03
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-04
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-05
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-06
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-07
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-08
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-09
BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-10
```

GitHub 自动提供的 `Source code.zip` 和 `Source code.tar.gz` 只是源码，不是可安装应用。

## 合并 DMG

将全部分卷放在同一个文件夹中。假设文件位于 macOS 的“下载”文件夹，打开“终端”并执行：

```bash
cd ~/Downloads

cat BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg.part-* \
  > BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg
```

如果浏览器修改了文件名，请先恢复原始名称，确保编号从 `00` 连续到 `10`。

## 验证完整性

合并后执行：

```bash
shasum -a 256 BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg
```

v0.2.0 正确的 SHA-256 为：

```text
4be71aa52d66b931ca583bab797ca8505d43137e33fe2f666b9fbbeb4611cf45
```

如果结果不同，请不要安装，并重新下载缺失或损坏的分卷。Release 附件中的 `SHA256SUMS.txt` 还包含每个分卷各自的校验值。

## 安装与首次启动

1. 双击合并后的 `BanG-Voice-Studio-0.2.0-Apple-Silicon.dmg`。
2. 将 `BanG Voice Studio.app` 拖入“应用程序”文件夹。
3. 第一次启动时，在“应用程序”中右键点击 BanG Voice Studio。
4. 选择“打开”，然后在系统提示中再次选择“打开”。

当前版本尚未经过 Apple Developer ID 公证，因此第一次启动需要使用右键“打开”。

## 主要功能

- BanG Dream! 角色声线选择与搜索
- 角色参考语音试听
- 中文及日文台词生成
- 语速与情感随机度调整
- 本地生成记录
- WAV 音频试听与导出
- 全程本地推理

首次生成和切换角色时需要加载模型，等待时间会比连续使用同一角色更长。生成过程中请不要强制退出应用。

## 从源码运行

仓库不包含模型、参考语音、Python 虚拟环境或构建产物。开发者需要自行准备：

```text
models/GPT_weights_v2ProPlus/
models/SoVITS_weights_v2ProPlus/
models/pretrained_models/
角色语音/<乐队>/<角色>/
```

安装环境并启动：

```bash
chmod +x scripts/*.sh
./scripts/setup-macos.sh
npm install
npm run tauri:dev
```

## 使用声明

本项目是非官方本地工具，与 Bushiroad、BanG Dream! Project 及相关权利方无关。请勿将生成内容用于冒充、欺骗、骚扰或未经授权的商业用途，并确保你对使用和分发的素材拥有相应权利。
