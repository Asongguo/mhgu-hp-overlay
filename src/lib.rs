#![no_std]

extern crate alloc;

pub mod buffer;
pub mod config;
pub mod font;
pub mod monster;
pub mod names;
pub mod oe;
pub mod render;
pub mod menu;

use core::fmt::Write;

use crate::buffer::StringBuffer;
use crate::config::{CONFIG, load_ini_config};
use crate::font::ensure_chinese_font_loaded;
use crate::monster::{
    MAX_POINTERS_IN_LIST, MonsterDisplay, MonsterPointerList, PartDisplay,
    is_monster_data_sane, is_valid_monster_ptr, is_valid_pointer_list,
    scan_monster_pointer_list_chunk,
};
use crate::render::{draw_debug_hud, draw_hp_overlay};

pub static mut CACHED_LIST_PTR: *const MonsterPointerList = core::ptr::null();
static mut SCAN_COOLDOWN: u32 = 0;

#[forge::entry]
fn main() {
    load_ini_config();
}

// ==========================================
// 游戏画面 HUD 叠加渲染回调
// ==========================================

#[forge::imgui_free_render]
fn render_overlay() {
    unsafe {
        if !CONFIG.enabled {
            return;
        }

        // 0. 确保中文字体（根据配置大小）已加载
        ensure_chinese_font_loaded(CONFIG.font_size);

        let heap = forge::mem::heap_addr();

        // 1. 获取/刷新列表指针 (分帧平摊切片扫描，彻底杜绝单帧遍历 5MB 导致掉帧)
        if CACHED_LIST_PTR.is_null() || !is_valid_pointer_list(CACHED_LIST_PTR) {
            CACHED_LIST_PTR = core::ptr::null();
            if SCAN_COOLDOWN == 0 {
                let (found_ptr, cycle_finished) = scan_monster_pointer_list_chunk();
                if !found_ptr.is_null() {
                    CACHED_LIST_PTR = found_ptr;
                    SCAN_COOLDOWN = 0;
                } else if cycle_finished {
                    // 整轮扫描完毕未找到，进入 60 帧冷却后再开始下一轮
                    SCAN_COOLDOWN = 60;
                }
            } else {
                SCAN_COOLDOWN -= 1;
            }
        }

        // 2. 读取怪物并过滤
        let default_part = PartDisplay {
            stagger_count: 0,
            break_count: 0,
            stagger_hp: 0,
            break_hp: 0,
            active: false,
        };
        let mut list: [MonsterDisplay; MAX_POINTERS_IN_LIST] = [
            MonsterDisplay {
                id: 0,
                hp: 0,
                max_hp: 0,
                in_current_map: false,
                parts: [default_part; 8],
                part_count: 0,
            };
            MAX_POINTERS_IN_LIST
        ];
        let mut valid_count = 0;

        if !CACHED_LIST_PTR.is_null() {
            let count = core::ptr::read_unaligned(core::ptr::addr_of!((*CACHED_LIST_PTR).count));
            let check_len = (count as usize).min(MAX_POINTERS_IN_LIST);
            for i in 0..check_len {
                let m_ptr = core::ptr::read_unaligned(core::ptr::addr_of!((*CACHED_LIST_PTR).monsters[i]));
                if !is_valid_monster_ptr(m_ptr) {
                    continue;
                }

                if is_monster_data_sane(m_ptr) {
                    let location_flag = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).location_flag));
                    let identifier2 = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).identifier2)) as u32;
                    let identifier1 = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).identifier1)) as u32;
                    let id = (identifier1 << 8) | identifier2;
                    let hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).hp));
                    let max_hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).max_hp));

                    let in_current_map = location_flag == 0x4C;

                    let should_display = if CONFIG.current_map_only {
                        in_current_map && max_hp > 500
                    } else {
                        max_hp > 500
                    };

                    if should_display {
                        // 读取部位耐久数据 (8个部位)
                        let mut parts = [default_part; 8];
                        let mut part_count = 0;
                        for p_idx in 0..8 {
                            let part_ptr = core::ptr::addr_of!((*m_ptr).parts[p_idx]);
                            let stagger_count = core::ptr::read_unaligned(core::ptr::addr_of!((*part_ptr).stagger_count));
                            let break_count = core::ptr::read_unaligned(core::ptr::addr_of!((*part_ptr).break_count));
                            let stagger_hp = core::ptr::read_unaligned(core::ptr::addr_of!((*part_ptr).stagger_hp));
                            let break_hp = core::ptr::read_unaligned(core::ptr::addr_of!((*part_ptr).break_hp));

                            if stagger_hp > 0 || break_hp > 0 || stagger_count > 0 || break_count > 0 {
                                parts[p_idx] = PartDisplay {
                                    stagger_count,
                                    break_count,
                                    stagger_hp,
                                    break_hp,
                                    active: true,
                                };
                                part_count += 1;
                            }
                        }

                        list[valid_count] = MonsterDisplay {
                            id,
                            hp,
                            max_hp,
                            in_current_map,
                            parts,
                            part_count,
                        };
                        valid_count += 1;
                    }
                }
            }
        }

        let cfg = CONFIG;

        // 3. 诊断 HUD 渲染 (可选)
        if cfg.show_debug {
            // 在 debug 栏中打印当前血条使用的 (pos_x, pos_y) 坐标、尺寸及中文字体加载状态
            let mut info_buf = StringBuffer::<128>::new();
            let font_stat = crate::font::get_font_status();
            let _ = write!(
                info_buf,
                "Pos:({:.0},{:.0}) Size:({:.0}x{:.0}) Font:[{}]",
                cfg.pos_x, cfg.pos_y, cfg.bar_width, cfg.bar_height, font_stat
            );
            draw_debug_hud(heap, CACHED_LIST_PTR, SCAN_COOLDOWN, valid_count, Some(info_buf.as_str()));
        }

        // 4. 无怪物时的处理：若开启 debug 模式，显示占位血条以便用户校准位置
        if valid_count == 0 {
            if cfg.show_debug {
                // 设置为 500/1000，确保同时能看到绿条（剩余血量）和红条（损失血量）
                let mut dummy_parts = [default_part; 8];
                dummy_parts[0] = PartDisplay {
                    stagger_count: 0,
                    break_count: 0,
                    stagger_hp: 200,
                    break_hp: 350,
                    active: true,
                };
                dummy_parts[1] = PartDisplay {
                    stagger_count: 1,
                    break_count: 1,
                    stagger_hp: 120,
                    break_hp: 0,
                    active: true,
                };
                let dummy = [MonsterDisplay {
                    id: 0,
                    hp: 500,
                    max_hp: 1000,
                    in_current_map: true,
                    parts: dummy_parts,
                    part_count: 2,
                }];
                draw_hp_overlay(&dummy, &cfg);
            }
            return;
        }

        // 4. 按照总血量从大到小排序 (纵向排列)
        let active_slice = &mut list[..valid_count];
        for i in 0..active_slice.len() {
            for j in (i + 1)..active_slice.len() {
                if active_slice[j].max_hp > active_slice[i].max_hp {
                    active_slice.swap(i, j);
                }
            }
        }

        // 5. 屏幕 HUD 叠加绘制
        draw_hp_overlay(active_slice, &cfg);
    }
}