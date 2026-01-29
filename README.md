# handy-keys

Cross-platform global keyboard shortcuts library for Rust.

> **Fork Note**: This is a fork of [handy-computer/handy-keys](https://github.com/handy-computer/handy-keys) with added left/right modifier key support.

## Features

- **Cross-platform**: Works on macOS, Windows, and Linux
- **Global hotkeys**: Register system-wide keyboard shortcuts
- **Hotkey blocking**: Registered hotkeys are blocked from reaching other applications
- **Left/Right modifier detection**: Distinguish between left and right modifier keys (e.g., `LeftCmd` vs `RightCmd`)
- **Modifier-only hotkeys**: Support for shortcuts like `Cmd+Shift` without a key
- **String parsing**: Parse hotkeys from strings like `"Ctrl+Alt+Space"` or `"LeftCmd+K"`
- **Hotkey recording**: Low-level keyboard listener for "record a hotkey" UI flows
- **Serde support**: All types implement `Serialize`/`Deserialize`

## Installation

```toml
[dependencies]
handy-keys = { git = "https://github.com/hey-ella-ai/handy-keys", branch = "main" }
```

## Quick Start

```rust
use handy_keys::{HotkeyManager, Hotkey, Modifiers, Key};

fn main() -> handy_keys::Result<()> {
    let manager = HotkeyManager::new()?;

    // Register using the type-safe constructor
    let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::K)?;
    let id = manager.register(hotkey)?;

    // Or parse from a string
    let hotkey2: Hotkey = "Ctrl+Alt+Space".parse()?;
    manager.register(hotkey2)?;

    // Listen for events
    while let Ok(event) = manager.recv() {
        println!("Hotkey triggered: {:?}", event.id);
    }

    Ok(())
}
```

## Left/Right Modifier Support

This fork adds the ability to distinguish between left and right modifier keys.

### Generic vs Specific Modifiers

```rust
use handy_keys::{Hotkey, Modifiers, Key};

// Generic: matches EITHER left or right Command key
let hotkey: Hotkey = "Cmd+K".parse()?;
// Equivalent to:
let hotkey = Hotkey::new(Modifiers::CMD, Key::K)?;

// Specific: matches ONLY the left Command key
let hotkey: Hotkey = "LeftCmd+K".parse()?;
// Equivalent to:
let hotkey = Hotkey::new(Modifiers::CMD_LEFT, Key::K)?;

// Specific: matches ONLY the right Command key
let hotkey: Hotkey = "RightCmd+K".parse()?;
// Equivalent to:
let hotkey = Hotkey::new(Modifiers::CMD_RIGHT, Key::K)?;

// Mixed: left Command + right Shift + K
let hotkey: Hotkey = "LeftCmd+RightShift+K".parse()?;
```

### Modifier-Only Hotkeys

You can use specific modifier keys as hotkeys themselves:

```rust
// Right Option key as a hotkey trigger
let hotkey: Hotkey = "RightOpt".parse()?;
manager.register(hotkey)?;
```

### Recording with Left/Right Detection

When recording hotkeys, events now include specific left/right information:

```rust
use handy_keys::KeyboardListener;

let listener = KeyboardListener::new()?;

while let Ok(event) = listener.recv() {
    if event.is_key_down {
        // event.modifiers will show LeftCmd, RightShift, etc.
        println!("Modifiers: {}", event.modifiers);
        // Output: "LeftCmd+RightShift" (not just "Cmd+Shift")
    }
}
```

## Platform Notes

### macOS

Requires accessibility permissions. The library provides helpers to check and request access:

```rust
use handy_keys::{check_accessibility, open_accessibility_settings};

if !check_accessibility() {
    open_accessibility_settings()?;
}
```

### Windows

Uses low-level keyboard hooks. No special permissions required.

### Linux

Uses [rdev](https://crates.io/crates/rdev). On Wayland, hotkey blocking may not work due to compositor restrictions.

## Modifiers

### Generic Modifiers (match either side)

| Modifier | Aliases |
|----------|---------|
| `CMD` | `command`, `meta`, `super`, `win` |
| `CTRL` | `control` |
| `OPT` | `option`, `alt` |
| `SHIFT` | |
| `FN` | `function` (macOS only) |

### Left-Specific Modifiers

| Modifier | Aliases |
|----------|---------|
| `CMD_LEFT` | `leftcmd`, `leftcommand`, `lcmd`, `lcommand` |
| `CTRL_LEFT` | `leftctrl`, `leftcontrol`, `lctrl` |
| `OPT_LEFT` | `leftopt`, `leftoption`, `leftalt`, `lopt`, `lalt` |
| `SHIFT_LEFT` | `leftshift`, `lshift` |

### Right-Specific Modifiers

| Modifier | Aliases |
|----------|---------|
| `CMD_RIGHT` | `rightcmd`, `rightcommand`, `rcmd`, `rcommand` |
| `CTRL_RIGHT` | `rightctrl`, `rightcontrol`, `rctrl` |
| `OPT_RIGHT` | `rightopt`, `rightoption`, `rightalt`, `ropt`, `ralt` |
| `SHIFT_RIGHT` | `rightshift`, `rshift` |

## Recording Hotkeys

For implementing "press a key to set hotkey" UI:

```rust
use handy_keys::KeyboardListener;

let listener = KeyboardListener::new()?;

println!("Press a key combination...");
while let Ok(event) = listener.recv() {
    if event.is_key_down {
        if let Ok(hotkey) = event.as_hotkey() {
            println!("Recorded: {}", hotkey);
            break;
        }
    }
}
```

## Backward Compatibility

Existing code using generic modifiers (`Modifiers::CMD`, `"Cmd+K"`, etc.) continues to work unchanged. Generic modifiers match either left or right physical keys.

## License

MIT
