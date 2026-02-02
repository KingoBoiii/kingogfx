extern crate glfw;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KgfxEventKind {
  None = 0,
  Close = 1,
  Key = 2,
}

impl Default for KgfxEventKind {
  fn default() -> Self {
    KgfxEventKind::None
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
    KgfxKeyAction::Release
  }
}

impl From<glfw::Action> for KgfxKeyAction {
  fn from(a: glfw::Action) -> Self {
    match a {
      glfw::Action::Release => KgfxKeyAction::Release,
      glfw::Action::Press => KgfxKeyAction::Press,
      glfw::Action::Repeat => KgfxKeyAction::Repeat,
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
#[derive(Copy, Clone, Debug, Default)]
pub struct KgfxKeyEvent {
  pub key: i32,
  pub scancode: i32,
  pub action: KgfxKeyAction, // repr(C) enum, FFI-ok
  pub mods: i32,             // bitmask
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
      data: KgfxEventData { raw: KgfxEventRaw::default() },
    }
  }
}

// Rust-venlige accessors (ingen ABI-impact)
impl KgfxEvent {
  pub fn as_key(&self) -> Option<KgfxKeyEvent> {
    if self.kind == KgfxEventKind::Key {
      // SIKKERT så længe caller kun læser key når kind==Key
      Some(unsafe { self.data.key })
    } else {
      None
    }
  }

  pub fn as_raw(&self) -> KgfxEventRaw {
    unsafe { self.data.raw }
  }
}

pub(super) fn map_event(ev: glfw::WindowEvent) -> KgfxEvent {
  match ev {
    glfw::WindowEvent::Close => KgfxEvent {
      kind: KgfxEventKind::Close,
      data: KgfxEventData { raw: KgfxEventRaw::default() },
    },

    glfw::WindowEvent::Key(key, scancode, action, mods) => KgfxEvent {
      kind: KgfxEventKind::Key,
      data: KgfxEventData {
        key: KgfxKeyEvent {
          key: key as i32,
          scancode,
          action: KgfxKeyAction::from(action),
          mods: mods.bits() as i32,
        },
      },
    },

    _ => KgfxEvent::default(),
  }
}