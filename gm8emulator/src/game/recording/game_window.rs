use crate::{
    imgui,
    game::{
        Renderer,
        recording::{
            ContextMenu,
            window::{Window, DisplayInformation},
        },
    },
};

// for imgui callback
struct GameViewData {
    renderer: *mut Renderer,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

pub struct GameWindow {
    callback_data: GameViewData,
}

// Game window
impl Window for GameWindow {
    fn name(&self) -> String {
        "Game".to_owned()
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        if *info.game_running {
            self.display_window(info);
        } else {
            *info.setting_mouse_pos = false;
        }
    }

    fn is_open(&self) -> bool { true }
}


impl GameWindow {
    pub fn new() -> GameWindow {
        GameWindow {
            callback_data: GameViewData {
                w: 0,
                h: 0,
                x: 0,
                y: 0,
                renderer: std::ptr::null_mut(),
            },
        }
    }

    fn display_window(&mut self, info: &mut DisplayInformation) {
        if *info.setting_mouse_pos {
            info.frame.begin_screen_cover();
            info.frame.end();
            unsafe {
                cimgui_sys::igSetNextWindowCollapsed(false, 0);
                cimgui_sys::igSetNextWindowFocus();
            }
        }
        
        let (w, h) = info.game.renderer.stored_size();
        info.frame.setup_next_window(imgui::Vec2(f32::from(info.config.ui_width) - w as f32 - 8.0, 8.0), None, None);
        info.frame.begin_window(
            &format!("{}###{}", info.game.get_window_title(), self.name()),
            Some(imgui::Vec2(
                w as f32 + (2.0 * info.win_border_size),
                h as f32 + info.win_border_size + info.win_frame_height
            )),
            false,
            false,
            None,
        );
        let imgui::Vec2(x, y) = info.frame.window_position();
        self.callback_data = GameViewData {
            renderer: (&mut info.game.renderer) as *mut _,
            x: (x + info.win_border_size) as i32,
            y: (y + info.win_frame_height) as i32,
            w: w,
            h: h,
        };

        unsafe extern "C" fn callback(
            _draw_list: *const cimgui_sys::ImDrawList,
            ptr: *const cimgui_sys::ImDrawCmd
        ) {
            let data = &*((*ptr).UserCallbackData as *mut GameViewData);
            (*data.renderer).draw_stored(data.x, data.y, data.w, data.h);
        }
        
        if !info.frame.window_collapsed() {
            info.frame.callback(callback, &mut self.callback_data);
            
            if *info.setting_mouse_pos && info.frame.left_clicked() {
                *info.setting_mouse_pos = false;
                let imgui::Vec2(mouse_x, mouse_y) = info.frame.mouse_pos();
                *info.new_mouse_pos =
                    Some((-(x + info.win_border_size - mouse_x) as i32, -(y + info.win_frame_height - mouse_y) as i32));
            }
            
            if info.frame.window_hovered() && info.frame.right_clicked() {
                self.set_context_menu_instances(info);
            }
        }
        
        info.frame.end();
    }

    /// Gets all the instances the mouse is hovered over and puts them in a context menu
    fn set_context_menu_instances(&self, info: &mut DisplayInformation) {
        unsafe {
            cimgui_sys::igSetWindowFocusNil();
        }
        let offset = info.frame.window_position() + imgui::Vec2(info.win_border_size, info.win_frame_height);
        let imgui::Vec2(x, y) = info.frame.mouse_pos() - offset;
        let (x, y) = info.game.translate_screen_to_room(x as _, y as _);
        
        let mut options: Vec<(String, i32)> = Vec::new();
        let mut iter = info.game.room.instance_list.iter_by_drawing();
        while let Some(handle) = iter.next(&info.game.room.instance_list) {
            let instance = info.game.room.instance_list.get(handle);
            instance.update_bbox(info.game.get_instance_mask_sprite(handle));
            if x >= instance.bbox_left.get()
            && x <= instance.bbox_right.get()
            && y >= instance.bbox_top.get()
            && y <= instance.bbox_bottom.get()
            {
                use crate::game::GetAsset;
                let id = instance.id.get();
                let description = match info.game.assets.objects.get_asset(instance.object_index.get()) {
                    Some(obj) => format!("{} ({})", obj.name, id.to_string()),
                    None => format!("<deleted object> ({})", id.to_string()),
                };
                options.push((description, id));
            }
        }
        
        if options.len() > 0 {
            *info.context_menu = Some(ContextMenu::Instances { pos: info.frame.mouse_pos(), options });
        }
    }
}