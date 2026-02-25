use super::events::KeyAction;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Unknown = -1,

    Space = 32,
    Apostrophe = 39,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,

    Num0 = 48,
    Num1 = 49,
    Num2 = 50,
    Num3 = 51,
    Num4 = 52,
    Num5 = 53,
    Num6 = 54,
    Num7 = 55,
    Num8 = 56,
    Num9 = 57,

    Semicolon = 59,
    Equal = 61,

    A = 65, B = 66, C = 67, D = 68, E = 69, F = 70, G = 71, H = 72, I = 73, J = 74, K = 75, L = 76,
    M = 77, N = 78, O = 79, P = 80, Q = 81, R = 82, S = 83, T = 84, U = 85, V = 86, W = 87, X = 88,
    Y = 89, Z = 90,

    LeftBracket = 91,
    Backslash = 92,
    RightBracket = 93,
    GraveAccent = 96,
    World1 = 161,
    World2 = 162,

    Escape = 256,
    Enter = 257,
    Tab = 258,
    Backspace = 259,
    Insert = 260,
    Delete = 261,
    Right = 262,
    Left = 263,
    Down = 264,
    Up = 265,
    PageUp = 266,
    PageDown = 267,
    Home = 268,
    End = 269,

    CapsLock = 280,
    ScrollLock = 281,
    NumLock = 282,
    PrintScreen = 283,
    Pause = 284,

    F1 = 290, F2 = 291, F3 = 292, F4 = 293, F5 = 294, F6 = 295, F7 = 296, F8 = 297, F9 = 298, F10 = 299,
    F11 = 300, F12 = 301, F13 = 302, F14 = 303, F15 = 304, F16 = 305, F17 = 306, F18 = 307, F19 = 308,
    F20 = 309, F21 = 310, F22 = 311, F23 = 312, F24 = 313, F25 = 314,

    Kp0 = 320, Kp1 = 321, Kp2 = 322, Kp3 = 323, Kp4 = 324,
    Kp5 = 325, Kp6 = 326, Kp7 = 327, Kp8 = 328, Kp9 = 329,
    KpDecimal = 330,
    KpDivide = 331,
    KpMultiply = 332,
    KpSubtract = 333,
    KpAdd = 334,
    KpEnter = 335,
    KpEqual = 336,

    LeftShift = 340,
    LeftControl = 341,
    LeftAlt = 342,
    LeftSuper = 343,
    RightShift = 344,
    RightControl = 345,
    RightAlt = 346,
    RightSuper = 347,
    Menu = 348,
}

impl Key {
    pub const fn from_i32(value: i32) -> Option<Self> {
        use Key::*;
        Some(match value {
            -1 => Unknown,

            32 => Space, 39 => Apostrophe, 44 => Comma, 45 => Minus, 46 => Period, 47 => Slash,
            48 => Num0, 49 => Num1, 50 => Num2, 51 => Num3, 52 => Num4, 53 => Num5, 54 => Num6, 55 => Num7, 56 => Num8, 57 => Num9,
            59 => Semicolon, 61 => Equal,

            65 => A, 66 => B, 67 => C, 68 => D, 69 => E, 70 => F, 71 => G, 72 => H, 73 => I, 74 => J, 75 => K, 76 => L,
            77 => M, 78 => N, 79 => O, 80 => P, 81 => Q, 82 => R, 83 => S, 84 => T, 85 => U, 86 => V, 87 => W, 88 => X, 89 => Y, 90 => Z,

            91 => LeftBracket, 92 => Backslash, 93 => RightBracket, 96 => GraveAccent,
            161 => World1, 162 => World2,

            256 => Escape, 257 => Enter, 258 => Tab, 259 => Backspace, 260 => Insert, 261 => Delete,
            262 => Right, 263 => Left, 264 => Down, 265 => Up, 266 => PageUp, 267 => PageDown, 268 => Home, 269 => End,

            280 => CapsLock, 281 => ScrollLock, 282 => NumLock, 283 => PrintScreen, 284 => Pause,

            290 => F1, 291 => F2, 292 => F3, 293 => F4, 294 => F5, 295 => F6, 296 => F7, 297 => F8, 298 => F9, 299 => F10,
            300 => F11, 301 => F12, 302 => F13, 303 => F14, 304 => F15, 305 => F16, 306 => F17, 307 => F18, 308 => F19,
            309 => F20, 310 => F21, 311 => F22, 312 => F23, 313 => F24, 314 => F25,

            320 => Kp0, 321 => Kp1, 322 => Kp2, 323 => Kp3, 324 => Kp4, 325 => Kp5, 326 => Kp6, 327 => Kp7, 328 => Kp8, 329 => Kp9,
            330 => KpDecimal, 331 => KpDivide, 332 => KpMultiply, 333 => KpSubtract, 334 => KpAdd, 335 => KpEnter, 336 => KpEqual,

            340 => LeftShift, 341 => LeftControl, 342 => LeftAlt, 343 => LeftSuper,
            344 => RightShift, 345 => RightControl, 346 => RightAlt, 347 => RightSuper, 348 => Menu,

            _ => return None,
        })
    }

    pub const fn to_i32(self) -> i32 {
        self as i32
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers(pub i32);

impl KeyModifiers {
    pub const NONE: Self      = Self(0);
    pub const SHIFT: Self     = Self(0x0001);
    pub const CONTROL: Self   = Self(0x0002);
    pub const ALT: Self       = Self(0x0004);
    pub const SUPER: Self     = Self(0x0008);
    pub const CAPS_LOCK: Self = Self(0x0010);
    pub const NUM_LOCK: Self  = Self(0x0020);

    pub const fn from_i32(bits: i32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> i32 {
        self.0
    }

    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub action: KeyAction,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Default)]
pub struct Input {
}

impl Input {
    pub fn is_key_pressed(event: KeyEvent, key: Key) -> bool {
        event.key == key && event.action == KeyAction::Press
    }

    pub fn is_key_released(event: KeyEvent, key: Key) -> bool {
        event.key == key && event.action == KeyAction::Release
    }

    pub fn is_key_repeat(event: KeyEvent, key: Key) -> bool {
        event.key == key && event.action == KeyAction::Repeat
    }
}
