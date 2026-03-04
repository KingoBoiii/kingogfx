extern crate glfw;

use std::collections::VecDeque;
use std::ffi::c_void;

use glfw::Context;

use crate::window::backend::WindowBackend;
use crate::window::builder::WindowClientApi;
use crate::window::{KeyAction, KeyCode, KeyEvent, KeyModifiers, WindowError, WindowEvent};

pub(crate) struct WindowHandle {
    pub(crate) glfw: glfw::Glfw,
    pub(crate) window: glfw::PWindow,
    pub(crate) events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub(crate) event_queue: VecDeque<glfw::WindowEvent>,
}

impl WindowHandle {
    pub(crate) fn create(
        width: u32,
        height: u32,
        title: &str,
        client_api: WindowClientApi,
    ) -> Result<Self, WindowError> {
        let mut glfw = glfw::init(glfw::fail_on_errors).map_err(|_| WindowError::InitFailed)?;

        match client_api {
            WindowClientApi::OpenGl => {
                glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGl));
                glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
                glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
            }
            WindowClientApi::NoApi => {
                glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
            }
        }

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or_else(|| WindowError::CreateFailed {
                width,
                height,
                title: title.to_string(),
            })?;

        if client_api == WindowClientApi::OpenGl {
            window.make_current();
        }
        window.set_key_polling(true);
        window.set_close_polling(true);

        Ok(Self {
            glfw,
            window,
            events,
            event_queue: VecDeque::new(),
        })
    }

    pub(crate) fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    pub(crate) fn pop_raw_event(&mut self) -> Option<glfw::WindowEvent> {
        if self.event_queue.is_empty() {
            self.poll_events();
            for (_, ev) in glfw::flush_messages(&self.events) {
                self.event_queue.push_back(ev);
            }
        }

        self.event_queue.pop_front()
    }

    pub(crate) fn drain_raw_events(&mut self) -> Vec<glfw::WindowEvent> {
        self.poll_events();
        glfw::flush_messages(&self.events)
            .map(|(_, ev)| ev)
            .collect()
    }
}

fn map_key_action(a: glfw::Action) -> KeyAction {
    match a {
        glfw::Action::Release => KeyAction::Release,
        glfw::Action::Press => KeyAction::Press,
        glfw::Action::Repeat => KeyAction::Repeat,
    }
}

fn map_event(ev: glfw::WindowEvent) -> WindowEvent {
    match ev {
        glfw::WindowEvent::Close => WindowEvent::Close,
        glfw::WindowEvent::Key(key, _scancode, action, mods) => WindowEvent::Key(KeyEvent {
            key_code: KeyCode::from_i32(key as i32).unwrap_or(KeyCode::Unknown),
            action: map_key_action(action),
            modifiers: KeyModifiers::from_i32(mods.bits() as i32),
        }),
        _ => WindowEvent::Unknown,
    }
}

impl WindowBackend for WindowHandle {
    fn get_proc_address(&mut self, procname: &str) -> *const c_void {
        match self.window.get_proc_address(procname) {
            Some(p) => p as usize as *const c_void,
            None => std::ptr::null(),
        }
    }

    fn make_current(&mut self) {
        self.window.make_current();        
    }

    fn poll_event(&mut self) -> Option<WindowEvent> {
        self.pop_raw_event().map(map_event)
    }

    fn poll_events(&mut self) -> Vec<WindowEvent> {
        self.drain_raw_events().into_iter().map(map_event).collect()
    }

    fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    fn focus(&mut self) {
        self.window.focus();
    }

    fn should_close(&self) -> bool {
        self.window.should_close()
    }

    fn set_should_close(&mut self, value: bool) {
        self.window.set_should_close(value);
    }

    fn framebuffer_size(&self) -> (i32, i32) {
        self.window.get_framebuffer_size()
    }

    fn glfw_window_ptr(&mut self) -> *mut glfw_sys::GLFWwindow {
        self.window.window_ptr()
    }
}