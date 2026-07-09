# Agent Switch Tools v1.0.18 Release Notes

**Agent Switch Tools v1.0.18** fixes a long-standing annoyance on **Linux (Wayland)**: after the app was hidden to the tray and reopened, the window's native minimize/maximize/close buttons would sometimes stop responding to clicks. This release also adds a "Start minimized to tray" option for anyone who runs the app at login but doesn't want the dashboard popping up every time.

## What's New?

### 1. Fixed unresponsive window buttons on Linux/Wayland

On Wayland, GTK's native title bar buttons have a known bug: once a window is hidden and shown again — exactly what happens when you close the app to the tray and reopen it — the close/minimize/maximize buttons can stop reacting to clicks until the window is resized.

- The app now automatically "wakes up" the window controls every time the window regains focus, by briefly toggling the resizable property — the same effect as manually resizing the window, but invisible to you.
- This only runs on Linux; behavior on macOS and Windows is unchanged.
- Reference: [tauri-apps/tauri#11856](https://github.com/tauri-apps/tauri/issues/11856), [tauri-apps/tauri#13440](https://github.com/tauri-apps/tauri/issues/13440).

### 2. New: Start minimized to tray

A new toggle in **Settings → General → Startup**, right next to "Launch at login":

- When enabled, the app starts with only the tray icon visible — the dashboard window doesn't open automatically.
- Open the dashboard anytime from the tray menu ("Open Dashboard") or by launching the app again.
- **Off by default** — the dashboard still opens automatically on startup unless you turn this on.

---

## Notes

- The window-button fix only applies on Linux; if you're on macOS or Windows, no behavior changes.
- The "Start minimized to tray" setting takes effect from the next app launch.

_v1.0.18 makes the app's window controls reliable on Linux/Wayland and gives autostart users the option to stay out of the way until they need the dashboard._
