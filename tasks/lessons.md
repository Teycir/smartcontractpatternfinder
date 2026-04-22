# Lessons

## 2026-04-22

- A successful desktop package build is not the same thing as an installed launcher. When reporting desktop readiness, verify both the produced bundle artifacts and whether the user will actually see a launcher in the application menu or on the desktop.
- For local desktop apps that depend on an embedded HTTP service, never assume a common port like `8080` is safe. Use an app-specific port and verify the backend identity in the UI, otherwise a foreign local service can look "healthy" while breaking the app in confusing ways.
- If a desktop build depends on API keys, do not rely only on inherited shell environment variables. GUI launches need an explicit app-local config path, and the fallback behavior must tell the user exactly where to put the key file.

## 2026-04-23

- When a sibling repo already has a stable implementation of the same desktop feature, inspect it early and borrow its stability constraints before designing a fresh fix.
