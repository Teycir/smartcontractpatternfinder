# Tauri Desktop Work Plan

## Checklist

- [completed] Inspect the current web UI, the `secretscout` desktop reference, and repo-specific instructions
- [completed] Add desktop-aware runtime resolution for bundled templates and the bundled `scpf` CLI
- [completed] Create the Tauri desktop shell under `frontend/src-tauri`
- [completed] Redesign the frontend into a premium desktop workspace while preserving the existing scan workflow
- [completed] Verify frontend and Rust builds, then document the review outcome here

## Notes

- Goal: ship a Tauri desktop app with the same local-web workflow, but with a stronger desktop shell and styling closer to the `secretscout` UI language.
- Packaging constraint: `scpf-server` currently shells out through `cargo run`, so desktop mode needs a bundled CLI binary path.

## Review

- Frontend verification: `npm run build` completed successfully in `frontend/`.
- Rust verification: `cargo check -p scpf-server` completed successfully.
- Desktop verification: `cargo check --manifest-path frontend/src-tauri/Cargo.toml` completed successfully after staging Tauri resources locally in `frontend/src-tauri/resources/`.
- Desktop bundle status: `cargo tauri` is not installed in this environment yet, so a full packaged desktop build was not executed here.
- Packaging note: Tauri now stages templates plus any available `scpf-server` / `scpf` binaries into `frontend/src-tauri/resources/` during build, which avoids development-time failures from missing release artifacts.

---

# Desktop Build Repair - 2026-04-22

## Checklist

- [completed] Reproduce the current desktop build failure from the supported desktop build entrypoint
- [completed] Identify whether the blocker is missing tooling, Tauri configuration, or Rust/frontend build drift
- [completed] Implement the smallest clean fix required for a successful desktop build
- [completed] Re-run desktop build verification and capture the result

## Notes

- User request: "we want to build the desktop solution but failed"
- Reproduction command: `cd frontend && npm run desktop:build`
- The project build itself is healthy: frontend build, release Rust binaries, and Tauri packaging all succeed.
- The only reproduced failure was environment-specific: the AppImage stage failed inside the restricted sandbox with `failed to bundle project 'failed to run linuxdeploy'`.
- The same command succeeded outside the sandbox, so the repair here is documenting the required build environment rather than changing Tauri or Rust code.

## Review

- Reproduced failure in sandbox: `npm run desktop:build` failed during AppImage bundling after successfully producing the release app plus `.deb` and `.rpm` bundles.
- Verified fix path: reran `npm run desktop:build` outside the sandbox and it completed successfully.
- Verified desktop artifacts:
  - `frontend/src-tauri/target/release/bundle/deb/SCPF Desktop_0.1.0_amd64.deb`
  - `frontend/src-tauri/target/release/bundle/rpm/SCPF Desktop-0.1.0-1.x86_64.rpm`
  - `frontend/src-tauri/target/release/bundle/appimage/SCPF Desktop_0.1.0_amd64.AppImage`
- Documentation update: added a `Desktop Build` section to `README.md` with the supported build command, the required `tauri-cli`, the bundle output path, and the sandbox note for `linuxdeploy` / AppImage packaging.

---

# Desktop Shortcut Follow-Up - 2026-04-22

## Checklist

- [completed] Confirm whether the Linux desktop bundles already contain icons and desktop metadata
- [completed] Add a supported local shortcut flow so a completed build can create a visible launcher icon
- [completed] Verify the shortcut installer creates the expected application and desktop entries
- [completed] Document the shortcut flow and capture the lesson

## Notes

- User follow-up: "we do not see any desktop icon"
- Investigation focus: separate "bundle contains icons" from "launcher shortcut is installed in the user session".

## Review

- Verified bundle contents: the generated Linux bundles already include the launcher metadata plus icon assets under the expected freedesktop paths such as `usr/share/icons/hicolor/.../scpf-desktop.png`.
- Added `scripts/install_desktop_shortcut.sh`, which installs a user-local launcher, local icon assets, and a Desktop shortcut that target the built AppImage.
- Added `npm run desktop:install-shortcut` in `frontend/package.json` and documented the flow in `README.md`.
- Executed the installer successfully and verified:
  - `~/.local/share/applications/scpf-desktop.desktop`
  - `~/Desktop/SCPF Desktop.desktop`
  - `~/.local/share/icons/hicolor/{32x32,128x128,1024x1024}/apps/scpf-desktop.png`
  - `~/.local/bin/scpf-desktop-appimage` symlinked to the built AppImage

---

# Desktop Launch 422 Repair - 2026-04-22

## Checklist

- [completed] Reproduce the scan-launch 422 and identify whether the issue is request shape or runtime wiring
- [completed] Fix the desktop/frontend backend target so it does not collide with unrelated local services
- [completed] Add backend identity checks so the UI stops treating a foreign service as "Connected"
- [completed] Rebuild the desktop bundle, refresh the launcher, and verify the repaired launch path

## Notes

- User report: the desktop app showed `Request failed with status code 422` when pressing `Launch Scan`.
- Root cause: the UI was hardcoded to `127.0.0.1:8080`, but that port was already serving an unrelated local service. The SCPF desktop app therefore talked to the wrong backend entirely.
- Evidence:
  - `curl http://127.0.0.1:8080/api/status` returned a foreign schema (`last_run`, `current_repo`, `secrets_found`) instead of SCPF scan progress.
  - `curl http://127.0.0.1:8080/api/templates` returned a large secret-pattern registry instead of the SCPF template list.
- Repair strategy: move SCPF to a dedicated local port and validate that the backend identifies itself as `scpf-server`.

## Review

- Added shared frontend server config with SCPF origin `http://127.0.0.1:32145`.
- Updated:
  - `crates/scpf-server/src/main.rs` to bind to `SCPF_SERVER_ADDR` with default `127.0.0.1:32145` and to include `service: "scpf-server"` in health/status/templates responses.
  - `frontend/src-tauri/src/lib.rs` to start and stop the embedded server on the same dedicated address.
  - `frontend/src/utils/api.js` to reject foreign backend responses and surface plain-text 422 bodies cleanly.
  - `frontend/scripts/start-with-server.js` and `frontend/vite.config.js` to use the same dedicated SCPF backend origin without killing unrelated processes.
- Verification:
  - `npm run build` succeeded in `frontend/`.
  - `cargo check -p scpf-server` succeeded.
  - `cargo check --manifest-path frontend/src-tauri/Cargo.toml` succeeded.
  - `curl http://127.0.0.1:32145/api/health` returned `service: "scpf-server"`.
  - `curl http://127.0.0.1:32145/api/status` returned the expected SCPF scan schema.
  - `curl http://127.0.0.1:32145/api/templates` returned the expected SCPF template inventory.
  - `POST /api/start` on `127.0.0.1:32145` returned `{"message":"Scan started"}` instead of 422.
  - `npm run desktop:build` completed successfully and the launcher was refreshed with `scripts/install_desktop_shortcut.sh`.

---

# Desktop API Key Runtime Repair - 2026-04-22

## Checklist

- [completed] Confirm whether the desktop app can see any explorer API keys at runtime
- [completed] Add desktop-safe `.env` loading so the packaged app can read local API key files
- [completed] Improve the missing-key hint so 0-day-only fallback is explained clearly
- [completed] Rebuild the desktop bundle and refresh the launcher

## Notes

- User report: the desktop run completed with only a 0-day summary.
- Runtime logs showed the scan engine was healthy but had no usable explorer keys:
  - `Failed: No API keys configured for ethereum`
  - `Failed: No API keys configured for polygon`
  - `Failed: No API keys configured for arbitrum`
- Current local state at investigation time:
  - no repo `.env`
  - no `ETHERSCAN_API_KEY` present in the current process environment
- Root cause: the packaged desktop app relied on environment variables and current-directory `.env` loading, which is fragile for GUI launches. The desktop path needed an explicit, app-local `.env` location.

## Review

- Added `crates/scpf-cli/src/runtime_env.rs` so the CLI now loads env files from:
  - current directory / parents via `dotenvy`
  - explicit `SCPF_ENV_FILE`
  - `SCPF_PROJECT_ROOT/.env`
  - `SCPF_RUNTIME_DIR/.env`
  - desktop-local config/data dirs such as `~/.local/share/com.teycir.scpf.desktop/.env`
  - ancestors of the `APPIMAGE` path when running from AppImage
- Updated `crates/scpf-cli/src/commands/scan.rs` to print an explicit API key setup hint when contract discovery falls back to 0-day-only mode because explorer keys are missing.
- Updated the desktop bundle build/startup path to:
  - ship `.env.example` in `frontend/src-tauri/resources/config/.env.example`
  - seed `~/.local/share/com.teycir.scpf.desktop/.env.example` on desktop runtime startup
  - pass `SCPF_RUNTIME_DIR`, `SCPF_PROJECT_ROOT`, and repo `.env` hints into the embedded server when available
- Verification:
  - `cargo check -p scpf-cli -p scpf-server` succeeded
  - `cargo check --manifest-path frontend/src-tauri/Cargo.toml` succeeded
  - targeted CLI proof with a temp `SCPF_RUNTIME_DIR/.env` no longer hit the `No API keys configured` branch
  - `npm run desktop:build` succeeded
  - launcher refreshed successfully
  - desktop-local sample file present at `~/.local/share/com.teycir.scpf.desktop/.env.example`

---

# Shared Runtime Env Encapsulation - 2026-04-22

## Checklist

- [completed] Inventory the current env/runtime loading flow across CLI, server, desktop, and frontend
- [completed] Extract a shared runtime config mechanism with encapsulated env discovery/loading
- [completed] Move CLI and desktop/backend consumers onto the shared mechanism
- [completed] Remove hardcoded frontend runtime assumptions for desktop backend configuration
- [completed] Add focused tests for the shared runtime config behavior
- [completed] Verify with cargo tests plus real CLI and desktop runs using the existing environment

## Notes

- Goal: keep env discovery, server address resolution, and API key loading in one reusable boundary instead of scattering them across CLI-only code and frontend constants.
- Constraint: browser code should not own desktop runtime env decisions; desktop should receive resolved runtime data from Rust.
- Shared boundary shipped as a new `crates/scpf-config` crate so CLI, server, and desktop can share env discovery, server endpoint resolution, and explorer API key loading.

## Review

- Added `crates/scpf-config` with:
  - shared env discovery / load order
  - shared `SCPF_SERVER_ADDR` resolution and `apiBaseUrl`
  - shared explorer API key loading
  - a serializable desktop/frontend runtime config snapshot
- Updated `scpf-cli` to consume the shared crate instead of local `runtime_env.rs` / `keys.rs`.
- Updated `scpf-server` to:
  - load env through the shared crate
  - bind through shared server config
  - expose `/api/runtime-config` for inspection
- Updated the Tauri desktop app to:
  - load env before startup
  - derive its backend address from the shared crate
  - expose `runtime_config` to the React app
- Updated the React frontend to resolve runtime config dynamically instead of hardcoding the backend origin in the browser bundle.
- Verification:
  - `cargo check -p scpf-config -p scpf-cli -p scpf-server` succeeded
  - `npm run build` in `frontend/` succeeded
  - `cargo check --manifest-path frontend/src-tauri/Cargo.toml` succeeded outside sandbox
  - `cargo test -p scpf-config -p scpf-cli -p scpf-server` succeeded outside sandbox
  - live CLI run succeeded:
    `cargo run --release -p scpf-cli --bin scpf -- scan --pages 0 --fetch-zero-day 1 --extract-sources 0`
    and generated `/home/teycir/smartcontractpatternfinderReports/report_1776886028/0day_summary.md`
  - live desktop run succeeded on an alternate env-driven port to prove the frontend is no longer pinned to `32145`:
    - `/api/runtime-config` returned `apiBaseUrl: http://127.0.0.1:32148`
    - desktop scan completed successfully and wrote `/home/teycir/.local/share/com.teycir.scpf.desktop/reports/report_1776888224/0day_summary.md`
- Residual caveat:
  - the desktop `/api/export` route still returned `No report available` for the 0-day-only flow even though the report file was created and the SSE logs showed the final `0-Day summary` line.
  - I hardened the parser for `0day_summary.md` and ANSI-stripped log lines, but the live desktop export path still needs one more pass if we want `/api/export` to surface 0-day-only summaries reliably.

---

# Desktop Black Screen Stability Repair - 2026-04-23

## Checklist

- [completed] Inspect the desktop frontend/server code paths plus prior desktop repair notes
- [completed] Document the likely failure modes and the intended repair approach here before editing
- [completed] Reduce backend SSE/log-stream leak pressure during long desktop runs
- [completed] Reduce frontend console render churn during long-running scans
- [completed] Add crash containment so desktop runtime failures surface as a recoverable screen instead of a black window
- [completed] Verify with frontend build plus Rust checks/tests and capture the outcome

## Notes

- User report: "scpf becomes unstable on desktop, we become dark screen after some time"
- Working hypothesis from code inspection:
  - the desktop console keeps rendering a growing live log list while scans stream frequent output
  - the server keeps every historical SSE sender in memory and never prunes disconnected clients
  - a frontend runtime exception currently has no error boundary, so a fatal render/effect failure can collapse into an empty dark shell
- Repair goal: keep the desktop UI responsive during long scans and make any remaining failure recoverable and diagnosable instead of presenting a black screen.
- `secretscout` was used as a stability reference for the console path, especially its tighter log-pressure expectations for desktop UX.

## Review

- Backend hardening:
  - updated `crates/scpf-server/src/main.rs` so the SSE sender list prunes disconnected clients instead of retaining stale channels forever
- Frontend hardening:
  - updated `frontend/src/components/Console.jsx` to batch log commits, defer rendered log updates, cap the rendered viewport to the latest 250 lines, and isolate the console from parent rerenders with `React.memo`
  - reduced avoidable parent churn in `frontend/src/components/Scanner.jsx` by skipping config/progress state updates when polled values are unchanged
  - added Tauri-only CSS fallbacks in the frontend so desktop uses more stable non-blurred surfaces and fewer infinite animations
- Crash containment:
  - added `frontend/src/components/DesktopCrashBoundary.jsx` plus matching CSS and wrapped the app in `frontend/src/main.jsx` so unexpected frontend/runtime failures surface as a recoverable error screen instead of a blank dark window
- Verification:
  - `npm run build` in `frontend/` succeeded
  - `cargo check -p scpf-server` succeeded
  - `cargo check --manifest-path frontend/src-tauri/Cargo.toml` succeeded
  - `cargo test -p scpf-server` succeeded outside sandbox
