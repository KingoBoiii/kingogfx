use crate::window::{KgfxKey, KgfxKeyEvent, KgfxKeyModifiers};

extern crate glfw;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Release,
    Press,
    Repeat,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: i32,
    pub scancode: i32,
    pub action: KeyAction,
    pub mods: i32,
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
    Key = 2,
}

impl Default for KgfxEventKind {
    fn default() -> Self {
        Self::None
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KgfxKeyAction {
    Release = 0,
    Press = 1,
    Repeat = 2,
}

impl Default for KgfxKeyAction {
    fn default() -> Self {
        Self::Release
    }
}

impl From<KeyAction> for KgfxKeyAction {
    fn from(a: KeyAction) -> Self {
        match a {
            KeyAction::Release => Self::Release,
            KeyAction::Press => Self::Press,
            KeyAction::Repeat => Self::Repeat,
        }
    }
}

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
    pub key: KgfxKeyEvent,
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

impl From<KeyEvent> for KgfxKeyEvent {
    fn from(k: KeyEvent) -> Self {
        Self {
            key: KgfxKey::from_i32(k.key).unwrap_or(KgfxKey::Unknown),
            action: k.action.into(),
            modifiers: KgfxKeyModifiers::from_i32(k.mods),
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
                kind: KgfxEventKind::Key,
                data: KgfxEventData { key: k.into() },
            },
            WindowEvent::Unknown => Self::default(),
        }
    }
}

impl KgfxEvent {
    pub fn as_key(&self) -> Option<KgfxKeyEvent> {
        if self.kind == KgfxEventKind::Key {
            Some(unsafe { self.data.key })
        } else {
            None
        }
    }
}
