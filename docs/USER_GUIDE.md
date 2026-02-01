# handy-keys User Guide

A practical guide to using the handy-keys library for global keyboard shortcuts in Rust.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Basic Usage](#basic-usage)
- [Left/Right Modifier Support](#leftright-modifier-support)
- [Recording Hotkeys](#recording-hotkeys)
- [Modifier-Only Hotkeys](#modifier-only-hotkeys)
- [String Parsing](#string-parsing)
- [Event Handling Patterns](#event-handling-patterns)
- [Platform-Specific Setup](#platform-specific-setup)
- [Common Use Cases](#common-use-cases)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

### Installation

Add handy-keys to your `Cargo.toml`:

```toml
[dependencies]
handy-keys = { git = "https://github.com/hey-ella-ai/handy-keys", branch = "main" }
```

### Minimal Example

```rust
use handy_keys::{HotkeyManager, Hotkey, Modifiers, Key, Result};

fn main() -> Result<()> {
    // Create the hotkey manager
    let manager = HotkeyManager::new()?;

    // Register a hotkey
    let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::K)?;
    let id = manager.register(hotkey)?;
    println!("Registered: {} (id: {:?})", hotkey, id);

    // Listen for events
    println!("Press Cmd+Shift+K to trigger the hotkey...");
    while let Ok(event) = manager.recv() {
        println!("Hotkey {:?}: {:?}", event.id, event.state);
    }

    Ok(())
}
```

---

## Basic Usage

### Creating a HotkeyManager

The `HotkeyManager` is the main entry point for registering and listening to hotkeys:

```rust
use handy_keys::HotkeyManager;

let manager = HotkeyManager::new()?;
```

On macOS, this will check for accessibility permissions. If not granted, it will fail with `Error::AccessibilityNotGranted`.

### Registering Hotkeys

There are two ways to create hotkeys:

**Method 1: Type-safe constructor**

```rust
use handy_keys::{Hotkey, Modifiers, Key};

// Single modifier + key
let hotkey = Hotkey::new(Modifiers::CMD, Key::K)?;

// Multiple modifiers + key
let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::K)?;

// Register it
let id = manager.register(hotkey)?;
```

**Method 2: String parsing**

```rust
let hotkey: Hotkey = "Cmd+Shift+K".parse()?;
let id = manager.register(hotkey)?;
```

### Listening for Events

**Blocking receive (recommended for dedicated threads):**

```rust
while let Ok(event) = manager.recv() {
    match event.state {
        HotkeyState::Pressed => println!("Hotkey pressed!"),
        HotkeyState::Released => println!("Hotkey released!"),
    }
}
```

**Non-blocking receive (for UI loops):**

```rust
loop {
    if let Some(event) = manager.try_recv() {
        println!("Hotkey event: {:?}", event);
    }

    // Do other work...
    std::thread::sleep(Duration::from_millis(10));
}
```

### Unregistering Hotkeys

```rust
manager.unregister(id)?;
```

---

## Left/Right Modifier Support

This fork adds the ability to distinguish between left and right modifier keys.

### Generic vs Specific Modifiers

**Generic modifiers** match EITHER left or right physical key:

```rust
// Matches either left or right Command key
let hotkey = Hotkey::new(Modifiers::CMD, Key::K)?;

// Also via string
let hotkey: Hotkey = "Cmd+K".parse()?;
```

**Specific modifiers** match ONLY the specified side:

```rust
// Only matches left Command key
let hotkey = Hotkey::new(Modifiers::CMD_LEFT, Key::K)?;

// Only matches right Command key
let hotkey = Hotkey::new(Modifiers::CMD_RIGHT, Key::K)?;

// Via string
let hotkey: Hotkey = "LeftCmd+K".parse()?;
let hotkey: Hotkey = "RightCmd+K".parse()?;
```

### Mixed Modifiers

You can mix generic and specific, or combine different sides:

```rust
// Left Command + Right Shift + K
let hotkey = Hotkey::new(
    Modifiers::CMD_LEFT | Modifiers::SHIFT_RIGHT,
    Key::K
)?;

// Or via string
let hotkey: Hotkey = "LeftCmd+RightShift+K".parse()?;
```

### Available Specific Modifiers

| Left | Right |
|------|-------|
| `CMD_LEFT` | `CMD_RIGHT` |
| `SHIFT_LEFT` | `SHIFT_RIGHT` |
| `CTRL_LEFT` | `CTRL_RIGHT` |
| `OPT_LEFT` | `OPT_RIGHT` |

### String Parsing Aliases

Left-specific:
- `LeftCmd`, `LCmd`, `LeftCommand`
- `LeftShift`, `LShift`
- `LeftCtrl`, `LCtrl`, `LeftControl`
- `LeftOpt`, `LOpt`, `LeftAlt`, `LAlt`

Right-specific:
- `RightCmd`, `RCmd`, `RightCommand`
- `RightShift`, `RShift`
- `RightCtrl`, `RCtrl`, `RightControl`
- `RightOpt`, `ROpt`, `RightAlt`, `RAlt`

---

## Recording Hotkeys

For implementing "press a key to set hotkey" UI flows, use `KeyboardListener`:

### Basic Recording

```rust
use handy_keys::KeyboardListener;

fn record_hotkey() -> handy_keys::Result<Hotkey> {
    let listener = KeyboardListener::new()?;

    println!("Press a key combination...");

    loop {
        if let Ok(event) = listener.recv() {
            if event.is_key_down {
                if let Ok(hotkey) = event.as_hotkey() {
                    println!("Recorded: {}", hotkey);
                    return Ok(hotkey);
                }
            }
        }
    }
}
```

### Recording with Escape to Cancel

```rust
use handy_keys::{KeyboardListener, Key};

fn record_hotkey_with_cancel() -> handy_keys::Result<Option<Hotkey>> {
    let listener = KeyboardListener::new()?;

    println!("Press a key combination (Escape to cancel)...");

    loop {
        if let Ok(event) = listener.recv() {
            if event.is_key_down {
                // Check for escape
                if event.key == Some(Key::Escape) {
                    return Ok(None);
                }

                if let Ok(hotkey) = event.as_hotkey() {
                    return Ok(Some(hotkey));
                }
            }
        }
    }
}
```

### Recording with Timeout

```rust
use handy_keys::{KeyboardListener, Error};
use std::time::Duration;

fn record_hotkey_with_timeout(timeout: Duration) -> handy_keys::Result<Option<Hotkey>> {
    let listener = KeyboardListener::new()?;
    let start = std::time::Instant::now();

    println!("Press a key combination...");

    while start.elapsed() < timeout {
        match listener.recv_timeout(Duration::from_millis(100)) {
            Ok(event) if event.is_key_down => {
                if let Ok(hotkey) = event.as_hotkey() {
                    return Ok(Some(hotkey));
                }
            }
            Err(Error::Timeout) => continue,
            _ => {}
        }
    }

    Ok(None) // Timed out
}
```

### Non-Blocking Recording (for UI loops)

```rust
use handy_keys::KeyboardListener;
use std::time::Duration;

let listener = KeyboardListener::new()?;

loop {
    // Check for keyboard events
    if let Some(event) = listener.try_recv() {
        if event.is_key_down {
            // Show the current combination being pressed
            println!("Current: {:?} + {:?}", event.modifiers, event.key);
        }
    }

    // Update UI, handle other events...
    std::thread::sleep(Duration::from_millis(10));
}
```

---

## Modifier-Only Hotkeys

You can register hotkeys that trigger on modifier keys alone:

### Single Modifier

```rust
// Function key alone (useful for push-to-talk)
let hotkey = Hotkey::new(Modifiers::FN, None)?;
manager.register(hotkey)?;

// Right Option key alone
let hotkey = Hotkey::new(Modifiers::OPT_RIGHT, None)?;
manager.register(hotkey)?;

// Via string
let hotkey: Hotkey = "Fn".parse()?;
let hotkey: Hotkey = "RightOpt".parse()?;
```

### Multiple Modifiers

```rust
// Cmd+Shift (without a key)
let hotkey = Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, None)?;
manager.register(hotkey)?;

// Via string
let hotkey: Hotkey = "Cmd+Shift".parse()?;
```

---

## String Parsing

### Modifier Aliases

| Modifier | Aliases |
|----------|---------|
| Command | `Cmd`, `Command`, `Meta`, `Super`, `Win`, `Windows` |
| Shift | `Shift` |
| Control | `Ctrl`, `Control` |
| Option/Alt | `Opt`, `Option`, `Alt` |
| Function | `Fn`, `Function` |

### Key Aliases

| Key | Aliases |
|-----|---------|
| Return | `Return`, `Enter` |
| Escape | `Escape`, `Esc` |
| Delete | `Delete`, `Backspace` |
| Space | `Space`, ` ` |
| Arrow keys | `Left`, `Right`, `Up`, `Down`, `LeftArrow`, etc. |

### Parsing Examples

```rust
// Standard hotkeys
let h: Hotkey = "Cmd+K".parse()?;
let h: Hotkey = "Ctrl+Alt+Delete".parse()?;
let h: Hotkey = "Shift+F5".parse()?;

// Left/right specific
let h: Hotkey = "LeftCmd+K".parse()?;
let h: Hotkey = "RightShift+Space".parse()?;
let h: Hotkey = "LCtrl+RAlt+Enter".parse()?;

// Key only
let h: Hotkey = "F1".parse()?;
let h: Hotkey = "Escape".parse()?;

// Modifier only
let h: Hotkey = "Fn".parse()?;
let h: Hotkey = "Cmd+Shift".parse()?;

// Case insensitive
let h: Hotkey = "CMD+SHIFT+K".parse()?;
let h: Hotkey = "cmd+shift+k".parse()?;
```

---

## Event Handling Patterns

### Dedicated Thread Pattern

Best for background hotkey listening:

```rust
use std::thread;
use handy_keys::{HotkeyManager, HotkeyEvent, HotkeyState};

fn start_hotkey_listener<F>(handler: F) -> thread::JoinHandle<()>
where
    F: Fn(HotkeyEvent) + Send + 'static,
{
    thread::spawn(move || {
        let manager = HotkeyManager::new().expect("Failed to create manager");

        // Register hotkeys...
        let hotkey: Hotkey = "Cmd+Space".parse().unwrap();
        manager.register(hotkey).unwrap();

        while let Ok(event) = manager.recv() {
            handler(event);
        }
    })
}

// Usage
let handle = start_hotkey_listener(|event| {
    println!("Hotkey event: {:?}", event);
});
```

### Channel-Based Pattern

For communicating between hotkey thread and main application:

```rust
use std::sync::mpsc;
use std::thread;

enum AppMessage {
    HotkeyPressed(HotkeyId),
    HotkeyReleased(HotkeyId),
    Quit,
}

fn main() -> handy_keys::Result<()> {
    let (tx, rx) = mpsc::channel();

    // Hotkey thread
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let manager = HotkeyManager::new().unwrap();
        let id = manager.register("Cmd+Space".parse().unwrap()).unwrap();

        while let Ok(event) = manager.recv() {
            let msg = match event.state {
                HotkeyState::Pressed => AppMessage::HotkeyPressed(event.id),
                HotkeyState::Released => AppMessage::HotkeyReleased(event.id),
            };
            if tx_clone.send(msg).is_err() {
                break;
            }
        }
    });

    // Main thread
    for msg in rx {
        match msg {
            AppMessage::HotkeyPressed(id) => println!("Pressed: {:?}", id),
            AppMessage::HotkeyReleased(id) => println!("Released: {:?}", id),
            AppMessage::Quit => break,
        }
    }

    Ok(())
}
```

### Push-to-Talk Pattern

```rust
use handy_keys::{HotkeyManager, HotkeyState, Modifiers};

fn main() -> handy_keys::Result<()> {
    let manager = HotkeyManager::new()?;

    // Register Fn key for push-to-talk
    let ptt_hotkey = Hotkey::new(Modifiers::FN, None)?;
    let ptt_id = manager.register(ptt_hotkey)?;

    println!("Push-to-talk ready. Hold Fn to talk...");

    while let Ok(event) = manager.recv() {
        if event.id == ptt_id {
            match event.state {
                HotkeyState::Pressed => {
                    println!("Recording started...");
                    // Start recording
                }
                HotkeyState::Released => {
                    println!("Recording stopped.");
                    // Stop recording
                }
            }
        }
    }

    Ok(())
}
```

---

## Platform-Specific Setup

### macOS

macOS requires accessibility permissions. Always check and handle this:

```rust
use handy_keys::{check_accessibility, open_accessibility_settings, HotkeyManager};

fn setup_hotkeys() -> handy_keys::Result<HotkeyManager> {
    // Check accessibility permission
    if !check_accessibility() {
        println!("Accessibility permission required!");
        println!("Opening System Settings...");
        open_accessibility_settings()?;

        return Err(handy_keys::Error::AccessibilityNotGranted);
    }

    // Permission granted, create manager
    HotkeyManager::new()
}
```

For GUI applications, you might want to poll for permission:

```rust
use std::time::Duration;
use std::thread;

fn wait_for_accessibility() {
    println!("Waiting for accessibility permission...");

    while !check_accessibility() {
        thread::sleep(Duration::from_secs(1));
    }

    println!("Permission granted!");
}
```

### Windows

No special permissions required. Just create the manager:

```rust
let manager = HotkeyManager::new()?;
```

### Linux

Works best on X11. On Wayland, hotkey blocking may not work:

```rust
let manager = HotkeyManager::new()?;

// Note: On Wayland, hotkeys will be detected but may not be blocked
// from reaching other applications
```

---

## Common Use Cases

### Configuration-Based Hotkeys

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct HotkeyConfig {
    push_to_talk: String,
    toggle_recording: String,
    cancel: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            push_to_talk: "Fn".to_string(),
            toggle_recording: "Cmd+Space".to_string(),
            cancel: "Escape".to_string(),
        }
    }
}

fn load_and_register_hotkeys(manager: &HotkeyManager, config: &HotkeyConfig) -> handy_keys::Result<()> {
    let ptt: Hotkey = config.push_to_talk.parse()?;
    manager.register(ptt)?;

    let toggle: Hotkey = config.toggle_recording.parse()?;
    manager.register(toggle)?;

    let cancel: Hotkey = config.cancel.parse()?;
    manager.register(cancel)?;

    Ok(())
}
```

### Multiple Hotkey Actions

```rust
use std::collections::HashMap;

enum Action {
    StartRecording,
    StopRecording,
    ToggleRecording,
    Cancel,
}

struct HotkeyController {
    manager: HotkeyManager,
    actions: HashMap<HotkeyId, Action>,
}

impl HotkeyController {
    fn new() -> handy_keys::Result<Self> {
        Ok(Self {
            manager: HotkeyManager::new()?,
            actions: HashMap::new(),
        })
    }

    fn register(&mut self, hotkey_str: &str, action: Action) -> handy_keys::Result<()> {
        let hotkey: Hotkey = hotkey_str.parse()?;
        let id = self.manager.register(hotkey)?;
        self.actions.insert(id, action);
        Ok(())
    }

    fn run<F>(&self, mut handler: F)
    where
        F: FnMut(&Action, HotkeyState),
    {
        while let Ok(event) = self.manager.recv() {
            if let Some(action) = self.actions.get(&event.id) {
                handler(action, event.state);
            }
        }
    }
}

// Usage
let mut controller = HotkeyController::new()?;
controller.register("Fn", Action::StartRecording)?;
controller.register("Cmd+Space", Action::ToggleRecording)?;
controller.register("Escape", Action::Cancel)?;

controller.run(|action, state| {
    if state == HotkeyState::Pressed {
        match action {
            Action::StartRecording => println!("Start"),
            Action::StopRecording => println!("Stop"),
            Action::ToggleRecording => println!("Toggle"),
            Action::Cancel => println!("Cancel"),
        }
    }
});
```

---

## Troubleshooting

### "Accessibility permission not granted" (macOS)

1. Open System Settings > Privacy & Security > Accessibility
2. Find your application in the list
3. Enable the toggle next to it
4. Restart your application

### Hotkey not triggering

1. **Check for conflicts**: Another application may have registered the same hotkey
2. **Verify registration**: Ensure `register()` returned `Ok`
3. **Check modifiers**: Make sure you're pressing the correct modifier keys
4. **Platform issues**: On Linux/Wayland, some hotkeys may not work

### Hotkey triggers twice

This can happen if you register the same hotkey twice. Check for duplicate registrations:

```rust
// This will fail with HotkeyAlreadyRegistered
let id1 = manager.register("Cmd+K".parse()?)?;
let id2 = manager.register("Cmd+K".parse()?)?; // Error!
```

### Left/right modifier not detected

Make sure you're using this fork, not the upstream library. The upstream library doesn't support left/right detection.

```toml
# Correct - use the fork
handy-keys = { git = "https://github.com/hey-ella-ai/handy-keys", branch = "main" }

# Wrong - upstream doesn't have left/right support
# handy-keys = "0.1"
```

### Event loop stops unexpectedly

The event loop stops when the `HotkeyManager` is dropped. Make sure the manager lives for the duration you need:

```rust
// Wrong - manager is dropped immediately
fn setup() {
    let manager = HotkeyManager::new().unwrap();
    // manager dropped here!
}

// Right - keep manager alive
fn main() {
    let manager = HotkeyManager::new().unwrap();
    // ... register hotkeys ...

    while let Ok(event) = manager.recv() {
        // Handle events
    }
    // manager lives until here
}
```

---

## Next Steps

- See the [API Reference](./API_REFERENCE.md) for complete type documentation
- Check the [examples](../examples/) directory for more code samples
- Report issues at https://github.com/hey-ella-ai/handy-keys/issues
