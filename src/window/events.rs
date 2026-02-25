use crate::window::KeyEvent;

extern crate glfw;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Release = 0,
    Press = 1,
    Repeat = 2,
}

impl Default for KeyAction {
    fn default() -> Self {
        Self::Release
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WindowEvent {
    Close,
    Key(KeyEvent),
    Unknown,
}

// -------- FFI ABI types --------

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KgfxEventKind {
    None = 0,
    Close = 1,
    KeyEvent = 2,
}

impl Default for KgfxEventKind {
    fn default() -> Self {
        Self::None
    }
}

pub type KgfxKeyAction = KeyAction;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct KgfxEventRaw {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union KgfxEventData {
    pub raw: KgfxEventRaw,
    pub key: KeyEvent,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KgfxEvent {
    pub kind: KgfxEventKind,
    pub data: KgfxEventData,
}

impl Default for KgfxEvent {
    fn default() -> Self {
        Self {
            kind: KgfxEventKind::None,
            data: KgfxEventData {
                raw: KgfxEventRaw::default(),
            },
        }
    }
}

impl From<WindowEvent> for KgfxEvent {
    fn from(value: WindowEvent) -> Self {
        match value {
            WindowEvent::Close => Self {
                kind: KgfxEventKind::Close,
                data: KgfxEventData {
                    raw: KgfxEventRaw::default(),
                },
            },
            WindowEvent::Key(k) => Self {
                kind: KgfxEventKind::KeyEvent,
                data: KgfxEventData { key: k },
            },
            WindowEvent::Unknown => Self::default(),
        }
    }
}

impl KgfxEvent {
    pub fn as_key(&self) -> Option<KeyEvent> {
        if self.kind == KgfxEventKind::KeyEvent {
            Some(unsafe { self.data.key })
        } else {
            None
        }
    }
}
