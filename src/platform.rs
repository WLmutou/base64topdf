/// 界面默认字体：不同系统自带的 CJK 字体不同。
#[cfg(target_os = "windows")]
pub(crate) const UI_FONT: &str = "Microsoft YaHei UI";

#[cfg(target_os = "macos")]
pub(crate) const UI_FONT: &str = "PingFang SC";

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub(crate) const UI_FONT: &str = "Noto Sans CJK SC";
