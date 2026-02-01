# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Another Yutto GUI is a Bilibili video downloader with a graphical interface built on Tauri 2 + Vue 3. It provides a GUI wrapper for [Yutto](https://github.com/yutto-dev/yutto), a Bilibili video downloader CLI tool.

**Tech Stack:**
- **Frontend**: Vue 3 + TypeScript + Vite + UnoCSS + Pinia
- **Backend**: Rust + Tauri 2
- **Build Tools**: PyInstaller (for bundling yutto), npm, cargo

## Development Commands

```bash
# Start development server (runs both Vite and Tauri)
npm run tauri dev

# Build frontend only
npm run build

# Fix linting issues
npm run lint:fix
```

### Building Bundled Binaries

The app can bundle yutto and ffmpeg binaries for standalone distribution:

1. **Build yutto executable:**
   ```bash
   cd build-scripts
   # Windows
   ./build-yutto.ps1
   # macOS/Linux
   ./build-yutto.sh
   ```

2. **Download ffmpeg binaries** and place in `binaries/` directory with platform-specific names, Do not download shared library package:
   - `ffmpeg-x86_64-pc-windows-msvc.exe`
   - `ffmpeg-x86_64-apple-darwin`
   - `ffmpeg-aarch64-apple-darwin`
   - `ffmpeg-x86_64-unknown-linux-gnu`

3. **Build the app:**
   before build, if ./binaries doesn`t have ffmpeg-x86_64-pc-* and yutto-x86_64-pc-*, you should build yutto first:
   ```bash
   cd build-scripts
   # Windows
   ./build-yutto.ps1
   # macOS/Linux
   ./build-yutto.sh
   ```
   Download ffmpeg at https://github.com/BtbN/FFmpeg-Builds/releases Do not download shared library package!
   The last step is
   ```bash
   npm run tauri build
   ```

## Architecture

### Frontend Architecture (Vue 3)

**Directory Structure:**
- `src/views/` - Main application views (Download, Queue, History, Settings, VideoPreview)
- `src/components/` - Reusable Vue components
- `src/stores/` - Pinia stores for state management (auth, download, history, queue, settings, titleBar, ui)
- `src/services/` - Service layer for API calls (api.ts, tauri.ts)
- `src/composables/` - Vue composables for reusable logic
- `src/router/` - Vue Router configuration
- `src/types/` - TypeScript type definitions
- `src/utils/` - Utility functions

**Key Frontend Patterns:**
- Uses Pinia for state management with persistence (`pinia-plugin-persistedstate`)
- Tauri commands are invoked via `@tauri-apps/api/core` in `src/services/tauri.ts`
- UnoCSS for utility-first styling with custom configuration in `uno.config.ts`
- Custom components follow a consistent pattern with props, emits, and composables

### Backend Architecture (Rust + Tauri)

**Directory Structure:**
- `src-tauri/src/commands/` - Tauri command handlers (video, download, auth, history, system)
- `src-tauri/src/services/` - Core business logic services
  - `bilibili_api/` - Bilibili API integration (video info, comments, quality, WBI auth)
  - `download_manager/` - Download queue management and yutto process control
  - `yutto_cli/` - Yutto CLI wrapper and output parsing
  - `storage.rs` - SQLite database operations
  - `process_control.rs` - Process pause/resume/kill operations
- `src-tauri/src/models/` - Data models (download, video, history, comment)
- `src-tauri/src/utils/` - Utility modules
  - `bundled_binaries.rs` - Handles bundled yutto/ffmpeg binary resolution
  - `quality_mapper.rs` - Maps quality codes to human-readable strings
  - `http_client.rs` - HTTP client utilities
  - `format.rs` - Formatting utilities

**Key Backend Patterns:**

1. **Download Manager (`services/download_manager/mod.rs`):**
   - Manages concurrent download queue with configurable max concurrent downloads
   - Spawns yutto processes and parses their output (both JSON and text formats)
   - Handles process lifecycle (start, pause, resume, cancel)
   - Emits progress events to frontend via Tauri events
   - Automatically starts pending downloads when slots become available

2. **Bundled Binaries (`utils/bundled_binaries.rs`):**
   - Implements fallback mechanism: bundled binaries → system binaries
   - Platform-specific binary naming (e.g., `yutto-x86_64-pc-windows-msvc.exe`)
   - Development mode support (loads from `binaries/` directory)
   - Automatically adds ffmpeg to PATH when available

3. **Storage (`services/storage.rs`):**
   - SQLite database for history and auth data
   - SESSDATA encryption using XOR cipher with machine-specific key
   - Automatic schema migration for database updates

4. **Process Control (`services/process_control.rs`):**
   - Platform-specific process pause/resume/kill operations
   - Windows: Uses `windows-sys` crate for process control
   - Unix: Uses `nix` crate for signal handling (SIGSTOP/SIGCONT/SIGKILL)

5. **Bilibili API (`services/bilibili_api/`):**
   - Implements WBI signature authentication for Bilibili API
   - Fetches video info, quality options, and comments
   - Handles VIP status and SESSDATA authentication

### State Management Flow

1. **Download Flow:**
   - User enters URL in `DownloadView.vue`
   - Frontend calls `getVideoInfo()` → Rust `fetch_video_info` command
   - User configures download options and clicks download
   - Frontend calls `startDownload()` → Rust `start_download` command
   - `DownloadManager` spawns yutto process and parses output
   - Progress updates emitted via Tauri events → Frontend updates `QueueStore`
   - On completion, task saved to history and removed from queue

2. **Authentication Flow:**
   - User opens Bilibili login window via `openBilibiliLogin()`
   - User logs in and copies SESSDATA cookie
   - SESSDATA saved to SQLite with machine-specific encryption
   - SESSDATA automatically included in API requests and yutto commands

### Important Implementation Details

**Yutto Output Parsing:**
- The app parses both JSON output (`--json-output` flag) and text output from yutto
- JSON parsing in `download_manager/json_parser.rs` for precise progress tracking
- Text parsing in `download_manager/output_parser.rs` for fallback and file path extraction
- File path extraction uses regex to find "已保存至" (saved to) messages

**Windows Encoding Handling:**
- Sets multiple environment variables to force UTF-8 encoding for yutto on Windows
- Uses `encoding_rs` crate to handle Windows console output encoding
- Disables colorama to prevent encoding issues

**Process Management:**
- Stores process ID (PID) in download task for pause/resume/cancel operations
- Checks if process is alive before attempting resume
- Kills process tree on cancel to ensure cleanup

**Comment Download:**
- Comments downloaded separately after video download completes
- Progress tracked independently with `comment_download_progress` field
- Saved to CSV file alongside video file

## Configuration Files

- `tauri.conf.json` - Tauri app configuration (window settings, bundle config, external binaries)
- `vite.config.ts` - Vite configuration with Tauri-specific settings (port 1420)
- `uno.config.ts` - UnoCSS configuration with custom presets
- `eslint.config.mjs` - ESLint configuration using @antfu/eslint-config
- `tsconfig.json` - TypeScript configuration
- `Cargo.toml` - Rust dependencies and build configuration

## Testing

User has manually performed debugging and no further action is required.

## Common Patterns

**Adding a new Tauri command:**
1. Define command handler in `src-tauri/src/commands/<module>.rs`
2. Add to `invoke_handler!` macro in `src-tauri/src/lib.rs`
3. Add TypeScript wrapper in `src/services/tauri.ts`
4. Use in Vue components via the service layer

**Adding a new Pinia store:**
1. Create store file in `src/stores/<name>.ts`
2. Use `defineStore` with composition API style
3. Add persistence if needed: `{ persist: true }`
4. Import and use in components

**Working with bundled binaries:**
- Development: Place binaries in `binaries/` directory with platform-specific names
- Production: Binaries bundled via `tauri.conf.json` `externalBin` field
- Access via `bundled_binaries::get_yutto_command_path()` with fallback to system version
