# rust版base64转pdf

界面用 iced，可将 Base64 字符串解码为图片，可选 PNG / JPG / JPEG / GIF，默认 PNG。

支持 Linux、Windows（macOS 也可直接构建）。

## 功能

- 粘贴 Base64 后自动解码，实时预览图片
- 支持 GIF 动图：预览区按原始帧延时循环播放，状态栏显示帧数
- 预览区支持左转 / 右转 90°，下载时保存旋转后的图片
- 自动识别图片类型（PNG / JPEG / GIF），显示 MIME 与文件大小
- 支持 `data:image/png;base64,...` 前缀、换行与空格、无填充及 URL-Safe Base64
- 支持打开 Base64 文本文件（自动忽略 UTF-8 BOM）
- 输出格式可选 PNG / JPG / JPEG / GIF，文件名后缀自动同步
- 下载时调用系统原生保存对话框；动图 GIF 未旋转时直接保存原始文件，旋转后逐帧重新编码保留动画

## 快速开始

```bash
cargo run --release
```

## 系统依赖

Linux 需要 GTK3、X11、Wayland 开发库（Windows 无需额外依赖）：

Ubuntu / Debian：

```bash
sudo apt-get install -y build-essential pkg-config \
  libgtk-3-dev libxkbcommon-dev libwayland-dev libx11-dev
```

CentOS / RedHat / Fedora：

```bash
sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config \
  gtk3-devel libxkbcommon-devel libX11-devel wayland-devel
```

界面为中文，若系统缺少中文字体会显示为方框，可安装 `fonts-noto-cjk`（Debian 系）或 `google-noto-sans-cjk-fonts`（RHEL 系）。

## 安装

### Linux

```bash
./install.sh              # 安装到 /usr/local（需要时自动使用 sudo）
./install.sh --user       # 安装到 ~/.local，无需 root
./install.sh --prefix /opt/base64topdf
./install.sh --no-deps    # 跳过系统依赖安装
./install.sh --uninstall  # 卸载
```

脚本会自动识别 apt-get / dnf / yum / microdnf，安装构建依赖，缺少 Rust 时通过 rustup 安装，并安装可执行文件、桌面入口与图标。

安装后可在应用菜单中找到「Base64 转图片」，或直接执行：

```bash
base64topdf
```

### Windows

安装 [Rust](https://rustup.rs/) 后：

```powershell
cargo build --release
.\target\release\base64topdf.exe
```

Windows 版使用 rfd 原生文件对话框，release 构建不会弹出控制台窗口。

## 项目结构

```
src/
├── main.rs      程序入口、窗口与主题配置
├── app.rs       应用状态、消息与业务逻辑
├── codec.rs     Base64 解码、图片识别与编码
├── view.rs      界面布局与样式
├── sample.rs    示例图片生成
└── platform.rs  各平台默认字体
assets/
└── base64topdf.svg   桌面图标
install.sh       Linux 安装脚本
```

## 技术说明

- GUI：iced 0.14，使用 tiny-skia 软件渲染后端
- 编解码：base64 0.22、image 0.25（仅启用 png / jpeg / gif）
- 动画：GIF 解码为完整合成帧与延时，通过 iced 订阅定时切换预览帧；0 或极小延时按 100ms 处理，避免刷新过快
- 文件对话框：rfd 0.15（Linux 使用 GTK3，Windows 使用原生 Win32）
