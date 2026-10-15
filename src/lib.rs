#![no_std]

extern crate alloc;

pub mod buffer;
pub mod config;
pub mod monster;
pub mod names;
pub mod render;

use core::fmt::Write;

use crate::buffer::StringBuffer;
use crate::config::{CONFIG, load_ini_config};
use crate::monster::{
    MAX_POINTERS_IN_LIST, MonsterDisplay, MonsterPointerList, find_monster_pointer_list,
    is_monster_data_sane, is_valid_pointer_list,
};
use crate::render::{draw_debug_hud, draw_hp_overlay};

static mut CACHED_LIST_PTR: *const MonsterPointerList = core::ptr::null();
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
        let heap = forge::mem::heap_addr();

        // 1. 获取/刷新列表指针
        if CACHED_LIST_PTR.is_null() || !is_valid_pointer_list(CACHED_LIST_PTR) {
            if SCAN_COOLDOWN == 0 {
                CACHED_LIST_PTR = find_monster_pointer_list();
                SCAN_COOLDOWN = 60; // 冷却 60 帧
            } else {
                SCAN_COOLDOWN -= 1;
            }
        }

        // 2. 读取怪物并过滤
        let mut list: [MonsterDisplay; MAX_POINTERS_IN_LIST] = [
            MonsterDisplay {
                id: 0,
                hp: 0,
                max_hp: 0,
                in_current_map: false,
            };
            MAX_POINTERS_IN_LIST
        ];
        let mut valid_count = 0;

        if !CACHED_LIST_PTR.is_null() {
            for i in 0..MAX_POINTERS_IN_LIST {
                let m_ptr = core::ptr::read_unaligned(core::ptr::addr_of!((*CACHED_LIST_PTR).monsters[i]));
                if m_ptr.is_null() {
                    break;
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
                        list[valid_count] = MonsterDisplay {
                            id,
                            hp,
                            max_hp,
                            in_current_map,
                        };
                        valid_count += 1;
                    }
                }
            }
        }

        let cfg = CONFIG;

        // 3. 诊断 HUD 渲染 (可选)
        if cfg.show_debug {
            // 在 debug 栏中打印当前血条使用的 (pos_x, pos_y) 坐标及尺寸，方便排查是否在屏幕外
            let mut info_buf = StringBuffer::<64>::new();
            let _ = write!(
                info_buf,
                "Pos:({:.0},{:.0}) Size:({:.0}x{:.0})",
                cfg.pos_x, cfg.pos_y, cfg.bar_width, cfg.bar_height
            );
            draw_debug_hud(heap, CACHED_LIST_PTR, SCAN_COOLDOWN, valid_count, Some(info_buf.as_str()));
        }

        // 4. 无怪物时的处理：若开启 debug 模式，显示占位血条以便用户校准位置
        if valid_count == 0 {
            if cfg.show_debug {
                // 设置为 500/1000，确保同时能看到绿条（剩余血量）和红条（损失血量）
                let dummy = [MonsterDisplay {
                    id: 0,
                    hp: 500,
                    max_hp: 1000,
                    in_current_map: true,
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