use core::fmt::Write;
use forge::imgui::{DrawList, color};
use forge::sys::imgui::{ImDrawFlags, ImVec2};

use crate::buffer::StringBuffer;
use crate::config::PluginConfig;
use crate::monster::{MonsterDisplay, MonsterPointerList};
use crate::names::{get_monster_cn_name, get_monster_en_name};

// ==========================================
// 绘制逻辑
// ==========================================

pub fn draw_outlined_text(
    draw_list: &DrawList,
    pos: ImVec2,
    text_col: forge::sys::imgui::ImU32,
    outline_col: forge::sys::imgui::ImU32,
    font_outline: bool,
    font_size: f32,
    text: &str,
) {
    let font_opt = unsafe { crate::font::CHINESE_FONT };

    let draw_single_text = |p: ImVec2, col: forge::sys::imgui::ImU32| {
        if let Some(ref font) = font_opt {
            draw_list.add_text_with_font(font, font_size, p, col, text);
        } else {
            draw_list.add_text(p, col, text);
        }
    };

    if font_outline {
        let offsets = [
            (-1.0, 0.0),
            (1.0, 0.0),
            (0.0, -1.0),
            (0.0, 1.0),
            (-1.0, -1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ];
        for (dx, dy) in offsets {
            draw_single_text(ImVec2::new(pos.x + dx, pos.y + dy), outline_col);
        }
    }

    draw_single_text(pos, text_col);
}

pub fn draw_debug_hud(
    _heap: u32,
    list_ptr: *const MonsterPointerList,
    cooldown: u32,
    valid_count: usize,
    extra_log: Option<&str>,
) {
    let draw_list = DrawList::foreground();
    let text_yellow = color(255, 230, 80, 240);
    let bg_color = color(0, 0, 0, 160);

    let mut buf = StringBuffer::<256>::new();
    let ptr_val = list_ptr as usize as u32;

    // 获取当前屏幕视口分辨率 (优先通过 ImGuiIO::DisplaySize 获取，比 Viewport 更通用稳定)
    let (scr_w, scr_h) = unsafe {
        let io = forge::sys::imgui::igGetIO_Nil();
        if !io.is_null() {
            ((*io).DisplaySize.x, (*io).DisplaySize.y)
        } else {
            (0.0, 0.0)
        }
    };

    if let Some(extra) = extra_log {
        let _ = write!(
            buf,
            "[DEBUG] Scr:{:.0}x{:.0} | Ptr:0x{:X} | Cd:{} | M:{} | {}",
            scr_w, scr_h, ptr_val, cooldown, valid_count, extra
        );
    } else {
        let _ = write!(
            buf,
            "[DEBUG] Scr:{:.0}x{:.0} | Ptr:0x{:X} | Cd:{} | M:{}",
            scr_w, scr_h, ptr_val, cooldown, valid_count
        );
    }

    // 每个英文字符宽约 7~8 像素，据此动态估算背景框宽度
    let text_str = buf.as_str();
    let estimated_width = (text_str.len() as f32 * 7.5 + 24.0).max(560.0);

    draw_list.add_rect_filled(
        ImVec2::new(8.0, 8.0),
        ImVec2::new(8.0 + estimated_width, 28.0),
        bg_color,
        2.0,
    );
    draw_list.add_text(ImVec2::new(12.0, 10.0), text_yellow, text_str);
}

pub fn draw_hp_overlay(monsters: &[MonsterDisplay], cfg: &PluginConfig) {
    let draw_list = DrawList::foreground();

    let start_x = cfg.pos_x;
    let mut current_y = cfg.pos_y;
    let bar_width = cfg.bar_width;
    let bar_height = cfg.bar_height;
    let row_spacing = cfg.row_spacing;

    let bg_color = color(cfg.bg_r, cfg.bg_g, cfg.bg_b, cfg.bg_a);
    let border_color = color(cfg.border_r, cfg.border_g, cfg.border_b, cfg.border_a);
    let red_lost = color(cfg.red_r, cfg.red_g, cfg.red_b, cfg.red_a);
    let green_remain = color(cfg.green_r, cfg.green_g, cfg.green_b, cfg.green_a);

    let text_color = color(cfg.font_color_r, cfg.font_color_g, cfg.font_color_b, cfg.font_color_a);
    let text_away_color = color(
        cfg.font_color_r.saturating_sub(60),
        cfg.font_color_g.saturating_sub(60),
        cfg.font_color_b.saturating_sub(60),
        cfg.font_color_a.saturating_sub(50).max(120),
    );
    let outline_color = color(
        cfg.outline_color_r,
        cfg.outline_color_g,
        cfg.outline_color_b,
        cfg.outline_color_a,
    );

    // MHGU 原生古代猎人图腾点阵数据 (16 行 × 12 列)
    // 位 11~0 分别代表第 0~11 列的点亮状态
    const TOTEM_PATTERN: [u16; 16] = [
        0xFF6, // row  0: ########.##.
        0xFF0, // row  1: ########....
        0xC30, // row  2: ##....##....
        0xDB6, // row  3: ##.##.##.##.
        0xCB6, // row  4: ##..#.##.##.
        0xFB0, // row  5: #####.##....
        0x030, // row  6: ......##....
        0x1B6, // row  7: ...##.##.##.
        0x1B6, // row  8: ...##.##.##.
        0x030, // row  9: ......##....
        0xFB0, // row 10: #####.##....
        0xCB6, // row 11: ##..#.##.##.
        0xDB6, // row 12: ##.##.##.##.
        0xC30, // row 13: ##....##....
        0xFF0, // row 14: ########....
        0xFF6, // row 15: ########.##.
    ];

    for m in monsters {
        let bar_top = current_y;
        let bar_bottom = bar_top + bar_height;

        if cfg.native_style {
            // ========================================================
            // 怪物猎人GU (MHGU) 原生点阵 UI 风格 (优化图腾渲染与防透色)
            // ========================================================
            let cell = bar_height / 16.0;

            // 图腾整个占 12 列宽 (包含 8 列主体 + 4 列齿孔间隙)
            let totem_w = cell * 12.0;
            let bar_actual_w = (bar_width - totem_w).max(40.0);

            // 划分血条色槽区域与图腾底板区域，严格无缝相接，互不溢入
            let (bar_start_x, bar_end_x, totem_left_x) = if cfg.mirror {
                // 图腾在最右侧: 血条在左 (start_x .. start_x + bar_actual_w), 图腾在右
                (start_x, start_x + bar_actual_w, start_x + bar_actual_w)
            } else {
                // 图腾在最左侧: 图腾在左 (start_x .. start_x + totem_w), 血条在右
                (start_x + totem_w, start_x + bar_width, start_x)
            };

            // 1. 图腾底板黑色基座 (彻底阻断绿红血色穿透齿孔)
            let totem_base_col = color(18, 20, 24, 230);
            draw_list.add_rect_filled(
                ImVec2::new(totem_left_x, bar_top),
                ImVec2::new(totem_left_x + totem_w, bar_bottom),
                totem_base_col,
                0.0,
            );

            // 2. 底层半透明黑灰背景 (仅在血条槽内)
            draw_list.add_rect_filled(
                ImVec2::new(bar_start_x, bar_top),
                ImVec2::new(bar_end_x, bar_bottom),
                bg_color,
                0.0,
            );

            // 3. 红色损失血量底槽 (全长)
            draw_list.add_rect_filled(
                ImVec2::new(bar_start_x, bar_top),
                ImVec2::new(bar_end_x, bar_bottom),
                red_lost,
                0.0,
            );

            // 4. 绿色当前剩余血量条 (支持 reverse_drain 配置扣血方向)
            let ratio = if m.max_hp > 0 {
                (m.hp as f32 / m.max_hp as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };

            // 扣减方向判定：
            // mirror=true: 默认满血靠右（图腾侧），从左向右扣
            // reverse_drain=true 则反转为满血靠左，向右扣
            let drain_from_right = if cfg.mirror {
                !cfg.reverse_drain
            } else {
                cfg.reverse_drain
            };

            let (green_left, green_right) = if drain_from_right {
                (bar_end_x - bar_actual_w * ratio, bar_end_x)
            } else {
                (bar_start_x, bar_start_x + bar_actual_w * ratio)
            };

            if ratio > 0.0 {
                draw_list.add_rect_filled(
                    ImVec2::new(green_left, bar_top),
                    ImVec2::new(green_right, bar_bottom),
                    green_remain,
                    0.0,
                );
            }

            // 5. 原生高光点缀：血条顶部细亮线 (增强层次感)
            let highlight_col = color(255, 255, 255, 45);
            draw_list.add_rect_filled(
                ImVec2::new(bar_start_x, bar_top),
                ImVec2::new(bar_end_x, bar_top + 1.2),
                highlight_col,
                0.0,
            );

            // 6. 文字描述 (置于血槽内，支持中英文切换)
            let mut text_buf = StringBuffer::<64>::new();
            let name = if cfg.use_english {
                get_monster_en_name(m.id)
            } else {
                get_monster_cn_name(m.id)
            };
            let _ = write!(text_buf, "{}: {}/{}", name, m.hp, m.max_hp);

            let label_color = if m.in_current_map { text_color } else { text_away_color };

            // 垂直居中微调 (结合字体行高和用户配置)
            let text_y = bar_top + (bar_height - cfg.font_size) * 0.5 + cfg.text_offset_y;
            let text_x = if cfg.mirror {
                start_x + 10.0
            } else {
                bar_start_x + 10.0
            };

            draw_outlined_text(
                &draw_list,
                ImVec2::new(text_x, text_y),
                label_color,
                outline_color,
                cfg.font_outline,
                cfg.font_size,
                text_buf.as_str(),
            );

            // 7. 绘制 MHGU 原生点阵图腾 (点间微隙 + 清晰分明无拉丝伪影)
            let totem_color = color(cfg.totem_r, cfg.totem_g, cfg.totem_b, cfg.totem_a);
            let dot_inset = 0.8f32; // 每个 LED 点缩进 0.8 像素，形成点阵间隙
            let dot_size = (cell - dot_inset).max(1.0);

            for row in 0..16 {
                let mask = TOTEM_PATTERN[row];
                let py = bar_top + (row as f32) * cell;

                for col in 0..12 {
                    // mask bit 11 对应 col 0, bit 0 对应 col 11
                    if (mask & (1 << (11 - col))) != 0 {
                        let px = if cfg.mirror {
                            // mirror: 图腾显示在右端，col 0 在最右侧，col 11 在左侧齿孔接缝
                            start_x + bar_width - ((col as f32) + 1.0) * cell
                        } else {
                            // 常规: 图腾显示在左端，col 0 在最左侧，col 11 在右侧齿孔接缝
                            start_x + (col as f32) * cell
                        };

                        // 点阵主色块 (每个点保留微隙，呈现原汁原味的游戏 LED 点阵屏质感)
                        draw_list.add_rect_filled(
                            ImVec2::new(px, py),
                            ImVec2::new(px + dot_size, py + dot_size),
                            totem_color,
                            0.0,
                        );
                    }
                }
            }
        } else {
            // ========================================================
            // 极简现代扁平血条风格 (带微圆角与完整描边)
            // ========================================================
            // 1. 底层纯黑半透明背景
            draw_list.add_rect_filled(
                ImVec2::new(start_x, bar_top),
                ImVec2::new(start_x + bar_width, bar_bottom),
                bg_color,
                3.0,
            );

            // 2. 红色损失血量底槽 (全长)
            draw_list.add_rect_filled(
                ImVec2::new(start_x, bar_top),
                ImVec2::new(start_x + bar_width, bar_bottom),
                red_lost,
                3.0,
            );

            // 3. 绿色当前剩余血量条 (支持 reverse_drain 反转)
            let ratio = if m.max_hp > 0 {
                (m.hp as f32 / m.max_hp as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let drain_from_right = if cfg.mirror {
                !cfg.reverse_drain
            } else {
                cfg.reverse_drain
            };

            let (green_left, green_right) = if drain_from_right {
                (start_x + bar_width * (1.0 - ratio), start_x + bar_width)
            } else {
                (start_x, start_x + bar_width * ratio)
            };

            if ratio > 0.0 {
                draw_list.add_rect_filled(
                    ImVec2::new(green_left, bar_top),
                    ImVec2::new(green_right, bar_bottom),
                    green_remain,
                    3.0,
                );
            }

            // 4. 外边框
            draw_list.add_rect(
                ImVec2::new(start_x, bar_top),
                ImVec2::new(start_x + bar_width, bar_bottom),
                border_color,
                3.0,
                1.2,
                ImDrawFlags::None,
            );

            // 5. 血条上层覆盖文字描述，支持中英文
            let mut text_buf = StringBuffer::<64>::new();
            let name = if cfg.use_english {
                get_monster_en_name(m.id)
            } else {
                get_monster_cn_name(m.id)
            };
            let _ = write!(text_buf, "{}: {}/{}", name, m.hp, m.max_hp);

            let label_color = if m.in_current_map { text_color } else { text_away_color };

            // 垂直居中于血条 (综合字体行高基准与大写字母光学中心，配合 cfg.text_offset_y 微调)
            let text_y = bar_top + (bar_height - cfg.font_size) * 0.5 + cfg.text_offset_y;
            let text_x = start_x + 8.0;

            draw_outlined_text(
                &draw_list,
                ImVec2::new(text_x, text_y),
                label_color,
                outline_color,
                cfg.font_outline,
                cfg.font_size,
                text_buf.as_str(),
            );
        }

        current_y += row_spacing;
    }
}
