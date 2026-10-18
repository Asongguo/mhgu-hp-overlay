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

    let mode = crate::oe::get_operation_mode();
    let mode_str = match mode {
        crate::oe::OperationMode::Docked => "Docked",
        crate::oe::OperationMode::Handheld => "Handheld",
    };

    if let Some(extra) = extra_log {
        let _ = write!(
            buf,
            "[DEBUG] Mode:{} Scr:{:.0}x{:.0} | Ptr:0x{:X} | Cd:{} | M:{} | {}",
            mode_str, scr_w, scr_h, ptr_val, cooldown, valid_count, extra
        );
    } else {
        let _ = write!(
            buf,
            "[DEBUG] Mode:{} Scr:{:.0}x{:.0} | Ptr:0x{:X} | Cd:{} | M:{}",
            mode_str, scr_w, scr_h, ptr_val, cooldown, valid_count
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

    // 通过 Switch Horizon OS 原生 nn::oe::GetOperationMode 获取当前模式缩放系数：
    // - 主机模式 (Docked): scale = 1.0 (按 1080p 原始基准布局)
    // - 掌机模式 (Handheld): scale = 720 / 1080 (~0.6667，按 720p 掌机等比缩小并保持相同屏幕占比)
    let scale = crate::oe::get_mode_scale();

    let start_x = cfg.pos_x * scale;
    let mut current_y = cfg.pos_y * scale;
    let bar_width = cfg.bar_width * scale;
    let bar_height = cfg.bar_height * scale;
    let row_spacing = cfg.row_spacing * scale;
    let font_size = cfg.font_size * scale;
    let text_offset_y = cfg.text_offset_y * scale;

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

            let totem_block_w = cell * 8.0;
            let bar_actual_w = (bar_width - totem_block_w).max(40.0);
            let (bar_start_x, bar_end_x) = if cfg.mirror {
                (start_x, start_x + bar_width - totem_block_w)
            } else {
                (start_x + totem_block_w, start_x + bar_width)
            };

            // 原生 UI 青蓝荧光色彩体系 (从配置读取 RGB 与 Alpha 透明度)
            let col_cyan_glow = color(cfg.totem_r, cfg.totem_g, cfg.totem_b, cfg.totem_a);
            let col_cyan_bg = color(cfg.bg_r, cfg.bg_g, cfg.bg_b, cfg.bg_a);

            // 1. 底框背景
            if cfg.bg_a > 0 {
                draw_list.add_rect_filled(
                    ImVec2::new(start_x, bar_top),
                    ImVec2::new(start_x + bar_width, bar_bottom),
                    col_cyan_bg,
                    0.0,
                );
            }

            // 2. 红色损失血量底槽 (扣血时显现)
            draw_list.add_rect_filled(
                ImVec2::new(bar_start_x, bar_top),
                ImVec2::new(bar_end_x, bar_bottom),
                red_lost,
                0.0,
            );

            // 3. 绿色当前剩余血量条 (支持 reverse_drain 配置扣血方向)
            if m.max_hp > 0 && m.hp > 0 {
                let ratio = (m.hp as f32 / m.max_hp as f32).clamp(0.0, 1.0);
                let current_w = bar_actual_w * ratio;

                let drain_from_right = if cfg.mirror {
                    !cfg.reverse_drain
                } else {
                    cfg.reverse_drain
                };

                let (green_left, green_right) = if drain_from_right {
                    (bar_end_x - current_w, bar_end_x)
                } else {
                    (bar_start_x, bar_start_x + current_w)
                };

                draw_list.add_rect_filled(
                    ImVec2::new(green_left, bar_top),
                    ImVec2::new(green_right, bar_bottom),
                    green_remain,
                    0.0,
                );
            }

            // 4. 点阵渲染：图腾与上下边框
            let total_cols = ((bar_width / cell) as usize).max(12);
            for r in 0..16 {
                let row_bits = TOTEM_PATTERN[r];
                let py = bar_top + (r as f32) * cell;

                if cfg.mirror {
                    // 镜像模式：
                    // - 右侧 12 列: 水平镜像图腾
                    for c in 0..12 {
                        if ((row_bits >> (11 - c)) & 1) != 0 {
                            let px = (start_x + bar_width) - ((c + 1) as f32) * cell;
                            draw_list.add_rect_filled(
                                ImVec2::new(px, py),
                                ImVec2::new(px + cell, py + cell),
                                col_cyan_glow,
                                0.0,
                            );
                        }
                    }
                    // - 左侧延伸血条部分 (上下边框点阵延伸至图腾方块，完全填补齿孔空隙)
                    if r == 0 || r == 15 {
                        let border_cols = total_cols.saturating_sub(8);
                        for c in 0..border_cols {
                            let px = start_x + (c as f32) * cell;
                            draw_list.add_rect_filled(
                                ImVec2::new(px, py),
                                ImVec2::new(px + cell, py + cell),
                                col_cyan_glow,
                                0.0,
                            );
                        }
                    }
                } else {
                    // 常规模式：
                    // - 左侧 12 列: 正常图腾图案
                    for c in 0..12 {
                        if ((row_bits >> (11 - c)) & 1) != 0 {
                            let px = start_x + (c as f32) * cell;
                            draw_list.add_rect_filled(
                                ImVec2::new(px, py),
                                ImVec2::new(px + cell, py + cell),
                                col_cyan_glow,
                                0.0,
                            );
                        }
                    }
                    // - 右侧延伸血条部分 (从左侧图腾方块外壁一直点亮至右端)
                    if r == 0 || r == 15 {
                        for c in 8..total_cols {
                            let px = start_x + (c as f32) * cell;
                            draw_list.add_rect_filled(
                                ImVec2::new(px, py),
                                ImVec2::new(px + cell, py + cell),
                                col_cyan_glow,
                                0.0,
                            );
                        }
                    }
                }
            }

            // 5. 文字描述 (置于血槽内，支持中英文切换)
            let mut text_buf = StringBuffer::<64>::new();
            let name = if cfg.use_english {
                get_monster_en_name(m.id)
            } else {
                get_monster_cn_name(m.id)
            };
            let _ = write!(text_buf, "{}: {}/{}", name, m.hp, m.max_hp);

            let label_color = if m.in_current_map { text_color } else { text_away_color };

            let text_y = bar_top + (bar_height - font_size) * 0.5 + text_offset_y;
            let text_x = if cfg.mirror {
                start_x + 8.0 * scale
            } else {
                bar_start_x + 8.0 * scale
            };

            draw_outlined_text(
                &draw_list,
                ImVec2::new(text_x, text_y),
                label_color,
                outline_color,
                cfg.font_outline,
                font_size,
                text_buf.as_str(),
            );
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

            // 垂直居中于血条 (综合字体行高基准与大写字母光学中心，配合 text_offset_y 微调)
            let text_y = bar_top + (bar_height - font_size) * 0.5 + text_offset_y;
            let text_x = start_x + 8.0 * scale;

            draw_outlined_text(
                &draw_list,
                ImVec2::new(text_x, text_y),
                label_color,
                outline_color,
                cfg.font_outline,
                font_size,
                text_buf.as_str(),
            );
        }

        current_y += row_spacing;
    }
}
