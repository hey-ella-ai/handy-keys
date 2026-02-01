# handy-keys API Reference

**Version**: 0.1.4-fork.1
**Repository**: https://github.com/hey-ella-ai/handy-keys
**License**: MIT

Cross-platform global keyboard shortcuts library for Rust with left/right modifier key support.

---

## Table of Contents

- [Installation](#installation)
- [Module Overview](#module-overview)
- [Core Types](#core-types)
  - [HotkeyManager](#hotkeymanager)
  - [KeyboardListener](#keyboardlistener)
  - [Hotkey](#hotkey)
  - [HotkeyId](#hotkeyid)
  - [HotkeyEvent](#hotkeyevent)
  - [HotkeyState](#hotkeystate)
  - [KeyEvent](#keyevent)
  - [Modifiers](#modifiers)
  - [ModifierType](#modifiertype)
  - [Key](#key)
- [Error Handling](#error-handling)
- [Platform Functions](#platform-functions)
- [Platform Notes](#platform-notes)

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
handy-keys = { git = "https://github.com/hey-ella-ai/handy-keys", branch = "main" }
```

---

## Module Overview

```
handy_keys
├── HotkeyManager      # High-level hotkey registration and event handling
├── KeyboardListener   # Low-level keyboard event streaming
├── Hotkey            # Hotkey definition (modifiers + optional key)
├── HotkeyId          # Unique identifier for registered hotkeys
├── HotkeyEvent       # Event emitted when a hotkey is triggered
├── HotkeyState       # Pressed or Released state
├── KeyEvent          # Raw keyboard event from listener
├── Modifiers         # Bitflags for modifier keys (with left/right support)
├── ModifierType      # Enum for querying modifier types
├── Key               # Keyboard keys and mouse buttons
├── Error             # Error types
└── Result<T>         # Type alias for Result<T, Error>
```

### Public Exports

```rust
pub use error::{Error, Result};
pub use listener::{BlockingHotkeys, KeyboardListener};
pub use manager::HotkeyManager;
pub use types::{Hotkey, HotkeyEvent, HotkeyId, HotkeyState, Key, KeyEvent, Modifiers};

#[cfg(target_os = "macos")]
pub use platform::macos::{check_accessibility, open_accessibility_settings};
```

---

## Core Types

### HotkeyManager

Platform-agnostic hotkey manager that wraps a `KeyboardListener` and filters events against registered hotkeys.

```rust
pub struct HotkeyManager { /* private fields */ }
```

#### Methods

##### `new() -> Result<Self>`

Create a new HotkeyManager.

- On macOS, checks for accessibility permissions and fails if not granted
- Registered hotkeys are blocked from reaching other applications
- On Linux/Wayland, blocking may not work due to compositor restrictions

```rust
let manager = HotkeyManager::new()?;
```

##### `register(&self, hotkey: Hotkey) -> Result<HotkeyId>`

Register a hotkey and return its unique ID.

- Returns `Error::HotkeyAlreadyRegistered` if the hotkey is already registered
- The hotkey will be blocked from reaching other applications

```rust
let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::K)?;
let id = manager.register(hotkey)?;
```

##### `unregister(&self, id: HotkeyId) -> Result<()>`

Unregister a hotkey by its ID.

- Returns `Error::HotkeyNotFound` if the ID is not found

```rust
manager.unregister(id)?;
```

##### `get_hotkey(&self, id: HotkeyId) -> Option<Hotkey>`

Get the hotkey definition associated with an ID.

```rust
if let Some(hotkey) = manager.get_hotkey(id) {
    println!("Hotkey: {}", hotkey);
}
```

##### `recv(&self) -> Result<HotkeyEvent>`

Blocking receive for hotkey events.

- Blocks until a hotkey event is received or the event loop stops
- Returns `Error::EventLoopNotRunning` if the manager is dropped

```rust
while let Ok(event) = manager.recv() {
    println!("Hotkey {:?} was {:?}", event.id, event.state);
}
```

##### `try_recv(&self) -> Option<HotkeyEvent>`

Non-blocking receive for hotkey events.

```rust
if let Some(event) = manager.try_recv() {
    println!("Hotkey triggered!");
}
```

##### `hotkey_count(&self) -> usize`

Get the number of currently registered hotkeys.

```rust
println!("Registered hotkeys: {}", manager.hotkey_count());
```

---

### KeyboardListener

Low-level keyboard listener that streams all keyboard events. Useful for implementing "record a hotkey" UI flows.

```rust
pub struct KeyboardListener { /* private fields */ }
```

#### Methods

##### `new() -> Result<Self>`

Create a new KeyboardListener in non-blocking mode.

- Events are observed but not blocked
- Use for "record hotkey" UI flows
- On macOS, checks for accessibility permissions

```rust
let listener = KeyboardListener::new()?;
```

##### `new_with_blocking(blocking_hotkeys: BlockingHotkeys) -> Result<Self>`

Create a new KeyboardListener with blocking support.

- Events matching hotkeys in the provided set will be blocked
- The set can be modified after creation

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

let blocking: BlockingHotkeys = Arc::new(Mutex::new(HashSet::new()));
let listener = KeyboardListener::new_with_blocking(blocking.clone())?;
```

##### `blocking_hotkeys(&self) -> Option<&BlockingHotkeys>`

Get a reference to the blocking hotkeys set (if blocking is enabled).

##### `recv(&self) -> Result<KeyEvent>`

Blocking receive for key events.

```rust
while let Ok(event) = listener.recv() {
    if event.is_key_down {
        println!("Key pressed: {:?}", event.key);
    }
}
```

##### `recv_timeout(&self, timeout: Duration) -> Result<KeyEvent>`

Blocking receive with timeout.

- Returns `Error::Timeout` if the timeout expires

```rust
use std::time::Duration;

match listener.recv_timeout(Duration::from_secs(5)) {
    Ok(event) => println!("Got event"),
    Err(Error::Timeout) => println!("Timed out"),
    Err(e) => eprintln!("Error: {}", e),
}
```

##### `try_recv(&self) -> Option<KeyEvent>`

Non-blocking receive for key events.

```rust
if let Some(event) = listener.try_recv() {
    println!("Event received");
}
```

---

### Hotkey

A hotkey definition consisting of modifiers and an optional key.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hotkey {
    pub modifiers: Modifiers,
    pub key: Option<Key>,
}
```

#### Methods

##### `new(modifiers: Modifiers, key: impl Into<Option<Key>>) -> Result<Self>`

Create a hotkey with modifiers and/or a key.

- At least one of modifiers or key must be provided
- Returns `Error::EmptyHotkey` if both are empty/None

```rust
// With modifiers and key
let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::K)?;

// Modifier-only
let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, None)?;

// Key-only
let hotkey = Hotkey::new(Modifiers::empty(), Key::F1)?;

// Left-specific modifier
let hotkey = Hotkey::new(Modifiers::CMD_LEFT, Key::K)?;
```

##### `to_lowercase_string(&self) -> String`

Format hotkey as lowercase string (e.g., "cmd+shift+k").

##### `to_handy_string(&self) -> String`

Format hotkey using platform-appropriate naming:
- macOS: "command", "option", "ctrl", "shift"
- Windows/Linux: "ctrl", "alt", "super", "shift"

#### Traits

##### `Display`

```rust
let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::K)?;
println!("{}", hotkey); // "Shift+Cmd+K"

let hotkey = Hotkey::new(Modifiers::CMD_LEFT, Key::K)?;
println!("{}", hotkey); // "LeftCmd+K"
```

##### `FromStr`

Parse a hotkey from a string like "Cmd+Shift+K" or "LeftCtrl+Space".

```rust
let hotkey: Hotkey = "Cmd+Shift+K".parse()?;
let hotkey: Hotkey = "LeftCmd+RightShift+K".parse()?;
let hotkey: Hotkey = "F1".parse()?;  // Key only
let hotkey: Hotkey = "Cmd+Shift".parse()?;  // Modifiers only
let hotkey: Hotkey = "RightOpt".parse()?;  // Modifier-only hotkey
```

---

### HotkeyId

A unique identifier for a registered hotkey.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HotkeyId(pub(crate) u32);
```

#### Methods

##### `as_u32(&self) -> u32`

Get the underlying u32 value.

---

### HotkeyEvent

Event emitted when a hotkey is pressed or released.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HotkeyEvent {
    pub id: HotkeyId,
    pub state: HotkeyState,
}
```

---

### HotkeyState

The state of a hotkey (pressed or released).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyState {
    Pressed,
    Released,
}
```

---

### KeyEvent

Event emitted during key recording.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeyEvent {
    pub modifiers: Modifiers,
    pub key: Option<Key>,
    pub is_key_down: bool,
    pub changed_modifier: Option<Modifiers>,
}
```

#### Fields

- `modifiers`: Currently held modifiers (with specific left/right info)
- `key`: The key that was pressed/released (None for modifier-only events)
- `is_key_down`: True for key press, false for key release
- `changed_modifier`: For modifier events, indicates which modifier changed

#### Methods

##### `as_hotkey(&self) -> Result<Hotkey>`

Convert this key event to a hotkey definition.

```rust
if let Ok(hotkey) = event.as_hotkey() {
    println!("Recorded hotkey: {}", hotkey);
}
```

---

### Modifiers

Bitflags for modifier keys with left/right support.

```rust
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Modifiers: u16 {
        // Left modifiers (bits 0-3)
        const CMD_LEFT = 1 << 0;
        const SHIFT_LEFT = 1 << 1;
        const CTRL_LEFT = 1 << 2;
        const OPT_LEFT = 1 << 3;

        // Right modifiers (bits 4-7)
        const CMD_RIGHT = 1 << 4;
        const SHIFT_RIGHT = 1 << 5;
        const CTRL_RIGHT = 1 << 6;
        const OPT_RIGHT = 1 << 7;

        // Function key (bit 8)
        const FN = 1 << 8;

        // Generic modifiers (match either side)
        const CMD = Self::CMD_LEFT.bits() | Self::CMD_RIGHT.bits();
        const SHIFT = Self::SHIFT_LEFT.bits() | Self::SHIFT_RIGHT.bits();
        const CTRL = Self::CTRL_LEFT.bits() | Self::CTRL_RIGHT.bits();
        const OPT = Self::OPT_LEFT.bits() | Self::OPT_RIGHT.bits();
    }
}
```

#### Constants

| Constant | Description |
|----------|-------------|
| `CMD_LEFT` | Left Command/Windows/Super key |
| `CMD_RIGHT` | Right Command/Windows/Super key |
| `CMD` | Either Command key (generic) |
| `SHIFT_LEFT` | Left Shift key |
| `SHIFT_RIGHT` | Right Shift key |
| `SHIFT` | Either Shift key (generic) |
| `CTRL_LEFT` | Left Control key |
| `CTRL_RIGHT` | Right Control key |
| `CTRL` | Either Control key (generic) |
| `OPT_LEFT` | Left Option/Alt key |
| `OPT_RIGHT` | Right Option/Alt key |
| `OPT` | Either Option/Alt key (generic) |
| `FN` | Function key (macOS) |

#### Methods

##### `contains_left(&self, modifier_type: ModifierType) -> bool`

Check if this modifier set contains a left-side modifier.

##### `contains_right(&self, modifier_type: ModifierType) -> bool`

Check if this modifier set contains a right-side modifier.

##### `is_side_specific(&self) -> bool`

Check if this modifier is a specific left or right variant (not generic).

##### `is_satisfied_by(&self, pressed: Modifiers) -> bool`

Check if the required modifiers are satisfied by the pressed modifiers.

- Generic modifiers (CMD, SHIFT, etc.) accept either left or right
- Specific modifiers (CMD_LEFT, SHIFT_RIGHT, etc.) require exact match

```rust
let required = Modifiers::CMD;  // Generic
assert!(required.is_satisfied_by(Modifiers::CMD_LEFT));   // true
assert!(required.is_satisfied_by(Modifiers::CMD_RIGHT));  // true

let required = Modifiers::CMD_LEFT;  // Specific
assert!(required.is_satisfied_by(Modifiers::CMD_LEFT));   // true
assert!(!required.is_satisfied_by(Modifiers::CMD_RIGHT)); // false
```

#### String Parsing Aliases

| Generic | Aliases |
|---------|---------|
| `CMD` | `command`, `meta`, `super`, `win`, `windows` |
| `SHIFT` | `shift` |
| `CTRL` | `control` |
| `OPT` | `option`, `alt` |
| `FN` | `function` |

| Left-Specific | Aliases |
|---------------|---------|
| `CMD_LEFT` | `leftcmd`, `leftcommand`, `lcmd`, `lcommand`, `lmeta`, `lsuper`, `lwin` |
| `SHIFT_LEFT` | `leftshift`, `lshift` |
| `CTRL_LEFT` | `leftctrl`, `leftcontrol`, `lctrl`, `lcontrol` |
| `OPT_LEFT` | `leftopt`, `leftoption`, `leftalt`, `lopt`, `lalt` |

| Right-Specific | Aliases |
|----------------|---------|
| `CMD_RIGHT` | `rightcmd`, `rightcommand`, `rcmd`, `rcommand`, `rmeta`, `rsuper`, `rwin` |
| `SHIFT_RIGHT` | `rightshift`, `rshift` |
| `CTRL_RIGHT` | `rightctrl`, `rightcontrol`, `rctrl`, `rcontrol` |
| `OPT_RIGHT` | `rightopt`, `rightoption`, `rightalt`, `ropt`, `ralt` |

---

### ModifierType

Enum for querying left/right variants.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierType {
    Cmd,
    Shift,
    Ctrl,
    Opt,
    Fn,
}
```

---

### Key

Keyboard keys and mouse buttons that can be used in hotkey combinations.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Key {
    // Letters: A-Z
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers: Num0-Num9
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    // Function keys: F1-F20
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    F11, F12, F13, F14, F15, F16, F17, F18, F19, F20,

    // Special keys
    Space, Return, Tab, Escape, Delete, ForwardDelete,
    Home, End, PageUp, PageDown,

    // Arrow keys
    LeftArrow, RightArrow, UpArrow, DownArrow,

    // Punctuation and symbols
    Minus, Equal, LeftBracket, RightBracket, Backslash,
    Semicolon, Quote, Comma, Period, Slash, Grave,

    // Keypad
    Keypad0, Keypad1, Keypad2, Keypad3, Keypad4,
    Keypad5, Keypad6, Keypad7, Keypad8, Keypad9,
    KeypadDecimal, KeypadMultiply, KeypadPlus, KeypadClear,
    KeypadDivide, KeypadEnter, KeypadMinus, KeypadEquals,

    // Lock keys
    CapsLock, ScrollLock, NumLock,

    // Mouse buttons
    MouseLeft, MouseRight, MouseMiddle, MouseX1, MouseX2,
}
```

#### String Parsing

Keys can be parsed from strings (case-insensitive):

```rust
let key: Key = "Space".parse()?;
let key: Key = "F1".parse()?;
let key: Key = "a".parse()?;
let key: Key = "Enter".parse()?;  // Alias for Return
let key: Key = "Esc".parse()?;    // Alias for Escape
```

---

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Accessibility permission not granted")]
    AccessibilityNotGranted,

    #[error("Failed to create event tap: {0}")]
    EventTapCreationFailed(String),

    #[error("Failed to create run loop source")]
    RunLoopSourceCreationFailed,

    #[error("Hotkey with id {0:?} not found")]
    HotkeyNotFound(HotkeyId),

    #[error("Hotkey already registered: {0}")]
    HotkeyAlreadyRegistered(String),

    #[error("Event loop not running")]
    EventLoopNotRunning,

    #[error("Operation timed out")]
    Timeout,

    #[error("Failed to start recording")]
    RecordingFailed,

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Hotkey cannot be empty")]
    EmptyHotkey,

    #[error("Invalid hotkey format: {0}")]
    InvalidHotkeyFormat(String),

    #[error("Unknown key: {0}")]
    UnknownKey(String),

    #[error("Unknown modifier: {0}")]
    UnknownModifier(String),

    #[error("Internal error: Mutex poisoned")]
    MutexPoisoned,
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Platform Functions

### macOS Only

##### `check_accessibility() -> bool`

Check if accessibility permissions are granted.

```rust
#[cfg(target_os = "macos")]
if !handy_keys::check_accessibility() {
    println!("Accessibility permission required");
}
```

##### `open_accessibility_settings() -> Result<()>`

Open the System Settings accessibility panel.

```rust
#[cfg(target_os = "macos")]
handy_keys::open_accessibility_settings()?;
```

---

## Platform Notes

### macOS

- Requires accessibility permissions
- Uses CGEventTap for event capture
- Full left/right modifier detection via keycode tracking
- FN key is supported
- Clean thread shutdown on drop

### Windows

- Uses low-level keyboard hooks
- No special permissions required
- Full left/right modifier detection
- Clean thread shutdown on drop

### Linux

- Uses [rdev](https://crates.io/crates/rdev)
- On Wayland, hotkey blocking may not work due to compositor restrictions
- Thread cleanup is limited (rdev grab blocks indefinitely)
- X11 provides better support for hotkey blocking

---

## Serde Support

All types implement `Serialize` and `Deserialize` for easy configuration storage:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Config {
    push_to_talk: Hotkey,
    toggle_recording: Hotkey,
}

let config = Config {
    push_to_talk: "Fn".parse()?,
    toggle_recording: "Cmd+Space".parse()?,
};

let json = serde_json::to_string(&config)?;
```
