use core::fmt::Write;

use forge::imgui::*;
use forge::sys::imgui::{ImGuiColorEditFlags, ImGuiInputTextFlags, ImVec2};

use crate::buffer::StringBuffer;
use crate::config::{
    CONFIG, PluginConfig, load_ini_config, save_ini_config,
};

static mut STATUS_MESSAGE: [u8; 128] = [0; 128];

fn set_status_message(msg: &str) {
    unsafe {
        let bytes = msg.as_bytes();
        let len = bytes.len().min(127);
        for i in 0..len {
            STATUS_MESSAGE[i] = bytes[i];
        }
        STATUS_MESSAGE[len] = 0;
    }
}

fn get_status_message() -> Option<&'static str> {
    unsafe {
        let ptr = core::ptr::addr_of!(STATUS_MESSAGE) as *const u8;
        if *ptr == 0 {
            return None;
        }
        let mut len = 0;
        while len < 128 && *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(ptr, len);
        core::str::from_utf8(slice).ok()
    }
}

#[inline]
fn u8_to_f32_rgba(r: u8, g: u8, b: u8, a: u8) -> [f32; 4] {
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
}

#[inline]
fn f32_to_u8_rgba(rgba: [f32; 4]) -> (u8, u8, u8, u8) {
    (
        (rgba[0] * 255.0 + 0.5) as u8,
        (rgba[1] * 255.0 + 0.5) as u8,
        (rgba[2] * 255.0 + 0.5) as u8,
        (rgba[3] * 255.0 + 0.5) as u8,
    )
}

// ==========================================
// Forge 菜单配置界面 (按下 Insert 键呼出)
// 挂载于主窗口 "Plugin UI" 折叠项下
// ==========================================

#[forge::imgui_render]
fn render_forge_menu() {
    unsafe {
        if tree_node(c"Monster HP Overlay") {
            let cfg = &mut *core::ptr::addr_of_mut!(CONFIG);

            // 1. 核心总开关
            checkbox(c"Show monster HP", &mut cfg.enabled);

            separator();

            // 2. 位置与尺寸调节
            input_float(c"Pos X", &mut cfg.pos_x, 1.0, 10.0, ImGuiInputTextFlags::None);
            input_float(c"Pos Y", &mut cfg.pos_y, 1.0, 10.0, ImGuiInputTextFlags::None);
            input_float(c"Bar width", &mut cfg.bar_width, 5.0, 20.0, ImGuiInputTextFlags::None);
            input_float(c"Bar height", &mut cfg.bar_height, 1.0, 5.0, ImGuiInputTextFlags::None);
            input_float(c"Row spacing", &mut cfg.row_spacing, 1.0, 5.0, ImGuiInputTextFlags::None);

            separator();

            // 3. 字体与文字设置
            input_float(c"Font size", &mut cfg.font_size, 0.5, 2.0, ImGuiInputTextFlags::None);
            input_float(c"Text offset Y", &mut cfg.text_offset_y, 0.5, 1.0, ImGuiInputTextFlags::None);
            checkbox(c"Font outline", &mut cfg.font_outline);

            let mut font_col = u8_to_f32_rgba(cfg.font_color_r, cfg.font_color_g, cfg.font_color_b, cfg.font_color_a);
            if color_edit4(c"Font color", &mut font_col, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(font_col);
                cfg.font_color_r = r;
                cfg.font_color_g = g;
                cfg.font_color_b = b;
                cfg.font_color_a = a;
            }

            let mut outline_col = u8_to_f32_rgba(cfg.outline_color_r, cfg.outline_color_g, cfg.outline_color_b, cfg.outline_color_a);
            if color_edit4(c"Outline color", &mut outline_col, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(outline_col);
                cfg.outline_color_r = r;
                cfg.outline_color_g = g;
                cfg.outline_color_b = b;
                cfg.outline_color_a = a;
            }

            let font_buf = &mut *core::ptr::addr_of_mut!(crate::config::CUSTOM_FONT_PATH);
            input_text_with_hint(
                c"Font TTF filename",
                c"default",
                font_buf,
                ImGuiInputTextFlags::None,
            );

            text("Appearance previews live. Font file changes need save and restart.");

            separator();

            // 4. 显示风格选项
            checkbox(c"Native Monster Hunter style bar", &mut cfg.native_style);
            checkbox(c"Mirror (Right-aligned / reverse fill)", &mut cfg.mirror);
            checkbox(c"Reverse drain animation", &mut cfg.reverse_drain);
            checkbox(c"English monster names", &mut cfg.use_english);
            checkbox(c"Current map only", &mut cfg.current_map_only);
            checkbox(c"Show monster parts durability", &mut cfg.show_parts);
            checkbox(c"Show debug info & layout preview", &mut cfg.show_debug);

            separator();

            // 5. 血条各类颜色配置
            let mut green = u8_to_f32_rgba(cfg.green_r, cfg.green_g, cfg.green_b, cfg.green_a);
            if color_edit4(c"Health bar color", &mut green, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(green);
                cfg.green_r = r;
                cfg.green_g = g;
                cfg.green_b = b;
                cfg.green_a = a;
            }

            let mut red = u8_to_f32_rgba(cfg.red_r, cfg.red_g, cfg.red_b, cfg.red_a);
            if color_edit4(c"Damage drain color", &mut red, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(red);
                cfg.red_r = r;
                cfg.red_g = g;
                cfg.red_b = b;
                cfg.red_a = a;
            }

            let mut bg = u8_to_f32_rgba(cfg.bg_r, cfg.bg_g, cfg.bg_b, cfg.bg_a);
            if color_edit4(c"Background color", &mut bg, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(bg);
                cfg.bg_r = r;
                cfg.bg_g = g;
                cfg.bg_b = b;
                cfg.bg_a = a;
            }

            let mut border = u8_to_f32_rgba(cfg.border_r, cfg.border_g, cfg.border_b, cfg.border_a);
            if color_edit4(c"Border color", &mut border, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(border);
                cfg.border_r = r;
                cfg.border_g = g;
                cfg.border_b = b;
                cfg.border_a = a;
            }

            let mut totem = u8_to_f32_rgba(cfg.totem_r, cfg.totem_g, cfg.totem_b, cfg.totem_a);
            if color_edit4(c"Totem icon color", &mut totem, ImGuiColorEditFlags::None) {
                let (r, g, b, a) = f32_to_u8_rgba(totem);
                cfg.totem_r = r;
                cfg.totem_g = g;
                cfg.totem_b = b;
                cfg.totem_a = a;
            }

            separator();

            // 6. 配置保存、还原与重置
            text("MHGU save: /hp_overlay.ini");

            if button(c"Save INI", ImVec2::new(80.0, 24.0)) {
                match save_ini_config() {
                    Ok(_) => set_status_message("Config saved to MHGU save: /hp_overlay.ini"),
                    Err(e) => set_status_message(e),
                }
            }
            same_line(0.0, 8.0);
            if button(c"Revert changes", ImVec2::new(110.0, 24.0)) {
                load_ini_config();
                set_status_message("Changes reverted from INI.");
            }
            same_line(0.0, 8.0);
            if button(c"Defaults", ImVec2::new(80.0, 24.0)) {
                *cfg = PluginConfig::default();
                crate::config::set_custom_font_path("LXGWWenKai-Regular.ttf");
                set_status_message("Defaults selected. Save INI to keep them.");
            }

            if let Some(msg) = get_status_message() {
                text(msg);
            }

            separator();

            // 7. 实时诊断状态信息
            let font_stat = crate::font::get_font_status();
            let mut diag_buf = StringBuffer::<128>::new();
            let _ = write!(
                diag_buf,
                "Font status: [{}] | List ptr: 0x{:08X}",
                font_stat,
                crate::CACHED_LIST_PTR as usize
            );
            text(diag_buf.as_str());

            tree_pop();
        }
    }
}
