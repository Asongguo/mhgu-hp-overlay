use core::ffi::CStr;
use core::ptr::addr_of_mut;
use forge::imgui::font::ImFont;
use forge::sys::imgui::{
    ImFontAtlas_GetGlyphRangesDefault, ImFontConfig_ImFontConfig,
    ImFontConfig_destroy, ImFontGlyphRangesBuilder_AddRanges, ImFontGlyphRangesBuilder_AddText,
    ImFontGlyphRangesBuilder_BuildRanges, ImFontGlyphRangesBuilder_Clear,
    ImFontGlyphRangesBuilder_ImFontGlyphRangesBuilder, ImFontGlyphRangesBuilder_destroy,
    ImVector_ImWchar, ImWchar, igGetIO_Nil,
};

use crate::names::ALL_CHINESE_CHARS;

pub static mut CHINESE_FONT: Option<ImFont> = None;
pub static mut FONT_STATUS_LOG: [u8; 64] = [0; 64];

const GLYPH_RANGES_CAPACITY: usize = 256;
static mut STATIC_GLYPH_RANGES: [ImWchar; GLYPH_RANGES_CAPACITY] = [0; GLYPH_RANGES_CAPACITY];
static mut RANGES_INITIALIZED: bool = false;
static mut FONT_TRIED_LOADING: bool = false;

pub fn get_font_status() -> &'static str {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(FONT_STATUS_LOG) as *const u8;
        let mut len = 0;
        while len < 64 && *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(ptr, len);
        core::str::from_utf8(slice).unwrap_or("ErrUTF8")
    }
}

/// 方案 B：使用 ImFontGlyphRangesBuilder 动态构建怪物血条所需字符集
/// 包括：ASCII 基础字符 + 冒号/斜杠/数字 + 所有怪物中文名涉及汉字
unsafe fn build_chinese_glyph_ranges() -> *const ImWchar {
    unsafe {
        let ranges_base = addr_of_mut!(STATIC_GLYPH_RANGES) as *mut ImWchar;

        if RANGES_INITIALIZED {
            return ranges_base as *const ImWchar;
        }

        let builder = ImFontGlyphRangesBuilder_ImFontGlyphRangesBuilder();
        ImFontGlyphRangesBuilder_Clear(builder);

        // 1. 添加 ImGui 默认基础字符范围 (ASCII 0x20 ~ 0xFF)
        let io = igGetIO_Nil();
        if !io.is_null() && !(*io).Fonts.is_null() {
            let default_ranges = ImFontAtlas_GetGlyphRangesDefault((*io).Fonts);
            if !default_ranges.is_null() {
                ImFontGlyphRangesBuilder_AddRanges(builder, default_ranges);
            }
        }

        // 2. 添加所有中文怪物名、标点、数字与状态提示所需字形
        let chars_bytes = ALL_CHINESE_CHARS.as_bytes();
        let text_ptr = chars_bytes.as_ptr() as *const core::ffi::c_char;
        let text_end = text_ptr.add(chars_bytes.len());
        ImFontGlyphRangesBuilder_AddText(builder, text_ptr, text_end);

        // 3. 构建码表范围到 ImVector 中
        let mut out_vec = ImVector_ImWchar {
            Size: 0,
            Capacity: 0,
            Data: core::ptr::null_mut(),
        };
        ImFontGlyphRangesBuilder_BuildRanges(builder, &mut out_vec);
        ImFontGlyphRangesBuilder_destroy(builder);

        // 4. 将 ImVector 中的数据转存到全局静态缓冲区 STATIC_GLYPH_RANGES 中 (使用裸指针写入避免引用 static mut)
        let count = (out_vec.Size as usize).min(GLYPH_RANGES_CAPACITY - 1);
        if !out_vec.Data.is_null() && count > 0 {
            for i in 0..count {
                *ranges_base.add(i) = *out_vec.Data.add(i);
            }
            *ranges_base.add(count) = 0; // 结尾终止符
        } else {
            // Fallback 最小码表 (基础拉丁 + 终止符0)
            *ranges_base.add(0) = 0x0020;
            *ranges_base.add(1) = 0x00FF;
            *ranges_base.add(2) = 0;
        }

        RANGES_INITIALIZED = true;
        ranges_base as *const ImWchar
    }
}

/// 尝试加载中文字体
pub unsafe fn ensure_chinese_font_loaded(font_size: f32) {
    unsafe {
        if FONT_TRIED_LOADING {
            return;
        }
        FONT_TRIED_LOADING = true;

        let romfs_mount = forge::fs::romfs::mount_point();

        // 仅在合法挂载的 romfs / 插件目录下安全探测字体文件，避免触发 nn::fs 的未挂载设备 PANIC
        let candidates = [
            alloc::format!("{}:/nativeNX/plugins/font.ttf", romfs_mount),
            alloc::format!("{}:/nativeNX/plugins/chinese.ttf", romfs_mount),
            alloc::format!("{}:/nativeNX/plugins/simhei.ttf", romfs_mount),
            alloc::format!("{}:/font.ttf", romfs_mount),
            alloc::format!("{}:/chinese.ttf", romfs_mount),
        ];

        let mut found_path: Option<alloc::string::String> = None;
        for path in &candidates {
            if forge::fs::file_exists(path) {
                found_path = Some(path.clone());
                break;
            }
        }

        let Some(valid_path) = found_path else {
            let msg = b"NoFontFound";
            for (i, &b) in msg.iter().enumerate() {
                FONT_STATUS_LOG[i] = b;
            }
            return;
        };

        let mut path_bytes = valid_path.into_bytes();
        path_bytes.push(0); // 确保以 NUL 结尾
        let c_path = CStr::from_bytes_with_nul_unchecked(&path_bytes);

        // 构建字形范围
        let ranges_ptr = build_chinese_glyph_ranges();

        // 初始化 ImFontConfig
        let cfg_ptr = ImFontConfig_ImFontConfig();
        if cfg_ptr.is_null() {
            let msg = b"CfgNull";
            for (i, &b) in msg.iter().enumerate() {
                FONT_STATUS_LOG[i] = b;
            }
            return;
        }

        // 调用原生 ImFontAtlas_AddFontFromFileTTF
        let io = igGetIO_Nil();
        if !io.is_null() && !(*io).Fonts.is_null() {
            let raw_font = forge::sys::imgui::ImFontAtlas_AddFontFromFileTTF(
                (*io).Fonts,
                c_path.as_ptr(),
                font_size,
                cfg_ptr,
                ranges_ptr,
            );
            if !raw_font.is_null() {
                CHINESE_FONT = Some(core::mem::transmute(raw_font));
                let msg = b"FontLoadedOK";
                for (i, &b) in msg.iter().enumerate() {
                    FONT_STATUS_LOG[i] = b;
                }
            } else {
                let msg = b"AddFontNull";
                for (i, &b) in msg.iter().enumerate() {
                    FONT_STATUS_LOG[i] = b;
                }
            }
        } else {
            let msg = b"IoOrFontsNull";
            for (i, &b) in msg.iter().enumerate() {
                FONT_STATUS_LOG[i] = b;
            }
        }

        ImFontConfig_destroy(cfg_ptr);
    }
}
