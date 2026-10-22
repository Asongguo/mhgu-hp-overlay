// ==========================================
// Nintendo Switch 屏幕模式自适应 (基于 ImGui 视口分辨率)
// ==========================================

use forge::sys::imgui::igGetIO_Nil;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OperationMode {
    Handheld = 0, // 掌机模式 (720p 布局)
    Docked = 1,   // 主机/底座模式 (1080p 布局)
}

/// 通过当前渲染视口尺寸 (DisplaySize) 获取运行模式，避免动态调用未导出的私有 SDK 符号
#[inline]
pub fn get_operation_mode() -> OperationMode {
    unsafe {
        let io = igGetIO_Nil();
        if !io.is_null() && (*io).DisplaySize.y >= 900.0 {
            OperationMode::Docked
        } else {
            OperationMode::Handheld
        }
    }
}

/// 是否处于主机/底座模式
#[inline]
pub fn is_docked() -> bool {
    get_operation_mode() == OperationMode::Docked
}

/// 计算当前模式相对于 1080p 基准配置的缩放系数：
/// - 主机模式 (1080p): scale = 1.0
/// - 掌机模式 (720p):  scale = 720.0 / 1080.0 (~0.66667)
#[inline]
pub fn get_mode_scale() -> f32 {
    unsafe {
        let io = igGetIO_Nil();
        if !io.is_null() && (*io).DisplaySize.y > 0.0 {
            let h = (*io).DisplaySize.y;
            if h >= 900.0 {
                1.0
            } else {
                (h / 1080.0).clamp(0.5, 1.0)
            }
        } else {
            match get_operation_mode() {
                OperationMode::Docked => 1.0,
                OperationMode::Handheld => 720.0 / 1080.0,
            }
        }
    }
}

