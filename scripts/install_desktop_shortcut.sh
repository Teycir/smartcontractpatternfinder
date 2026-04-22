#!/bin/sh

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
APPIMAGE_DIR="${REPO_ROOT}/frontend/src-tauri/target/release/bundle/appimage"
APPIMAGE_PATH="$(find "${APPIMAGE_DIR}" -maxdepth 1 -type f -name '*.AppImage' | sort | tail -n 1)"

if [ -z "${APPIMAGE_PATH}" ]; then
  echo "No AppImage build was found. Run 'cd frontend && npm run desktop:build' first." >&2
  exit 1
fi

LOCAL_BIN_DIR="${HOME}/.local/bin"
APPLICATIONS_DIR="${HOME}/.local/share/applications"
ICONS_DIR="${HOME}/.local/share/icons/hicolor"
DESKTOP_DIR="${HOME}/Desktop"
SYMLINK_PATH="${LOCAL_BIN_DIR}/scpf-desktop-appimage"
LAUNCHER_PATH="${APPLICATIONS_DIR}/scpf-desktop.desktop"
DESKTOP_LAUNCHER_PATH="${DESKTOP_DIR}/SCPF Desktop.desktop"

mkdir -p "${LOCAL_BIN_DIR}" "${APPLICATIONS_DIR}"
install -d "${ICONS_DIR}/32x32/apps" "${ICONS_DIR}/128x128/apps" "${ICONS_DIR}/1024x1024/apps"

chmod +x "${APPIMAGE_PATH}"
ln -sfn "${APPIMAGE_PATH}" "${SYMLINK_PATH}"

install -m 0644 "${REPO_ROOT}/frontend/src-tauri/icons/32x32.png" "${ICONS_DIR}/32x32/apps/scpf-desktop.png"
install -m 0644 "${REPO_ROOT}/frontend/src-tauri/icons/128x128.png" "${ICONS_DIR}/128x128/apps/scpf-desktop.png"
install -m 0644 "${REPO_ROOT}/frontend/src-tauri/icons/icon.png" "${ICONS_DIR}/1024x1024/apps/scpf-desktop.png"

cat > "${LAUNCHER_PATH}" <<EOF
[Desktop Entry]
Type=Application
Name=SCPF Desktop
Comment=Desktop workspace for Smart Contract Pattern Finder
Exec=${SYMLINK_PATH}
TryExec=${SYMLINK_PATH}
Icon=scpf-desktop
Terminal=false
Categories=Development;
StartupWMClass=scpf-desktop
EOF

chmod +x "${LAUNCHER_PATH}"

if [ -d "${DESKTOP_DIR}" ]; then
  install -m 0755 "${LAUNCHER_PATH}" "${DESKTOP_LAUNCHER_PATH}"
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APPLICATIONS_DIR}" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

echo "Installed launcher: ${LAUNCHER_PATH}"
if [ -d "${DESKTOP_DIR}" ]; then
  echo "Installed desktop shortcut: ${DESKTOP_LAUNCHER_PATH}"
fi
echo "Launcher target: ${APPIMAGE_PATH}"
