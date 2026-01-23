# Export/Import GUI Feature - Implementation Plan

## Overview
Add "Export config" and "Import config" menu entries with full-featured dialogs for managing espanso configuration transfer.

## Architecture Analysis

### Current Menu System
- **Location:** Menu entries defined in `espanso/src/cli/worker/builtin/`
- **Framework:** Built-in matches system with IDs ≥ 1,000,000,000
- **Menu Items Found:**
  - "Disable" / "Enable"
  - "Open search bar"
  - "Reload config" ← **Target location for new entries**
  - "Open config folder"
  - "Show logs"
  - "Exit espanso"
  - "Restart espanso"

### Current Dialog System (Modulo)
- **Location:** `espanso-modulo/` and `espanso/src/gui/modulo/`
- **Existing Types:**
  - `form` - Form inputs with fields
  - `search` - Search bar with items
  - `textview` - Display text/files
  - `welcome` - Welcome wizard
  - `troubleshoot` - Troubleshooting dialog
- **Architecture:** Rust backend + Native UI (C++/Objective-C/Win32)

### CLI Commands Integration
- **Export:** `espanso export [--scope <scopes>] [--wrap <width>]`
- **Import:** `espanso import [--scope <scopes>] [--yes] [--convert-lb]`
- **Location:** `espanso/src/cli/offline.rs`

## Implementation Plan

### Phase 1: Menu Entries (2 hours)
**Files to modify:**
- `espanso/src/cli/worker/builtin/mod.rs`
- `espanso/src/cli/worker/builtin/process.rs` (or new file)

**Tasks:**
1. Create `create_match_export_config()` function
2. Create `create_match_import_config()` function
3. Add to `get_builtin_matches()` list
4. Assign unique IDs (next available after restart/exit)

**Menu Structure:**
```
- Disable/Enable
- Open search bar
- Reload config
+ Export config          ← NEW
+ Import config          ← NEW
- Open config folder
- Show logs
---
- Exit espanso
- Restart espanso
```

### Phase 2: Export Dialog (3-4 hours)

#### 2.1 Backend (Rust)
**New file:** `espanso-modulo/src/sys/export/export.rs`

**Features:**
- Scope checkboxes (Config, Matches, Packages)
- Read-only textbox with export output
- "Copy to Clipboard" button
- "Save As" button (native file dialog)
- Dynamic update on scope change

**Data Flow:**
```
User clicks "Export config" 
  → Trigger builtin match
  → Launch modulo export dialog
  → Execute `espanso export --scope <selected>`
  → Display in textbox
  → User clicks Copy/Save
  → Close dialog
```

#### 2.2 Frontend (Native UI)
**Files to create:**
- `espanso-modulo/src/sys/export/export.cpp` (Windows)
- `espanso-modulo/src/sys/export/export.mm` (macOS)
- `espanso-modulo/src/sys/export/export_linux.cpp` (Linux - basic)

**UI Layout:**
```
┌─────────────────────────────────────┐
│ Export Espanso Configuration        │
├─────────────────────────────────────┤
│ Select scopes to export:            │
│ ☑ Config files                      │
│ ☑ Match files                       │
│ ☑ Package files                     │
├─────────────────────────────────────┤
│ ┌─────────────────────────────────┐ │
│ │ H4sIAAAAAAAA/+09XXPcRnKQ44tt... │ │
│ │ (base64 encoded export data)    │ │
│ │                                 │ │
│ └─────────────────────────────────┘ │
├─────────────────────────────────────┤
│  [Copy to Clipboard]  [Save As...]  │
│                          [Close]    │
└─────────────────────────────────────┘
```

### Phase 3: Import Dialog (4-5 hours)

#### 3.1 Backend (Rust)
**New file:** `espanso-modulo/src/sys/import/import.rs`

**Features:**
- Scope checkboxes (Config, Matches, Packages)
- Editable textbox for import code
- "Paste from Clipboard" button
- "Import from File" button (native file dialog)
- "Remove existing config files" checkbox
- "Check" button (validate + highlight scopes)
- "Import" button (with confirmation)
- "Cancel" button

**Validation Logic:**
```rust
fn validate_import_code(code: &str) -> Result<Vec<Scope>> {
    // 1. Try to decode base64
    // 2. Try to decompress gzip
    // 3. Try to read tar archive
    // 4. Detect which scopes are present
    // 5. Return list of detected scopes
}
```

**Data Flow:**
```
User clicks "Import config"
  → Launch modulo import dialog
  → User pastes/loads code
  → User clicks "Check"
    → Validate code
    → Highlight scope checkboxes (green=present, red=missing)
  → User clicks "Import"
    → Validate again
    → Show confirmation if "Remove existing" checked
    → Execute `espanso import --scope <selected> --yes`
    → Show success/error message
    → Close dialog
```

#### 3.2 Frontend (Native UI)
**Files to create:**
- `espanso-modulo/src/sys/import/import.cpp` (Windows)
- `espanso-modulo/src/sys/import/import.mm` (macOS)
- `espanso-modulo/src/sys/import/import_linux.cpp` (Linux - basic)

**UI Layout:**
```
┌─────────────────────────────────────┐
│ Import Espanso Configuration        │
├─────────────────────────────────────┤
│ Paste or load import code:          │
│ ┌─────────────────────────────────┐ │
│ │ H4sIAAAAAAAA/+09XXPcRnKQ44tt... │ │
│ │ (editable textbox)              │ │
│ │                                 │ │
│ └─────────────────────────────────┘ │
│  [Paste from Clipboard] [Load File] │
├─────────────────────────────────────┤
│ Select scopes to import:            │
│ ☑ Config files      (detected: ✓)  │
│ ☑ Match files       (detected: ✓)  │
│ ☑ Package files     (detected: ✗)  │
│                                     │
│ ☑ Remove existing config files      │
│   (only for imported scopes)        │
├─────────────────────────────────────┤
│  [Check]  [Import]  [Cancel]        │
└─────────────────────────────────────┘
```

### Phase 4: Integration & Testing (2-3 hours)

#### 4.1 Integration Points
1. **Menu → Dialog Launch**
   - `espanso/src/cli/worker/engine/funnel/ui.rs`
   - Handle new event types for export/import

2. **Dialog → CLI Commands**
   - Reuse existing `espanso/src/cli/offline.rs` functions
   - Create wrapper functions for modulo

3. **Error Handling**
   - Invalid import code
   - File I/O errors
   - Permission errors
   - Disk space errors

#### 4.2 Testing Checklist

**Export Dialog:**
- [ ] All scopes selected → full export
- [ ] Single scope selected → partial export
- [ ] Copy to clipboard works
- [ ] Save to file works
- [ ] Dynamic update on scope change
- [ ] Cancel closes dialog

**Import Dialog:**
- [ ] Paste from clipboard works
- [ ] Load from file works
- [ ] Check validates code correctly
- [ ] Scope detection highlights correctly
- [ ] Invalid code shows error
- [ ] Import without "Remove existing" works
- [ ] Import with "Remove existing" shows confirmation
- [ ] Confirmation shows correct directory path
- [ ] Import succeeds
- [ ] Cancel closes dialog

**Cross-Platform:**
- [ ] macOS: All features work
- [ ] Windows: All features work
- [ ] Linux: Basic functionality works

## File Structure

```
espanso/
├── src/
│   └── cli/
│       └── worker/
│           └── builtin/
│               ├── mod.rs              (modified)
│               └── config_transfer.rs  (new)
│
espanso-modulo/
├── src/
│   ├── sys/
│   │   ├── export/
│   │   │   ├── mod.rs                 (new)
│   │   │   ├── export.cpp             (new - Windows)
│   │   │   ├── export.mm              (new - macOS)
│   │   │   └── export_linux.cpp       (new - Linux)
│   │   └── import/
│   │       ├── mod.rs                 (new)
│   │       ├── import.cpp             (new - Windows)
│   │       ├── import.mm              (new - macOS)
│   │       └── import_linux.cpp       (new - Linux)
│   └── cli/
│       └── modulo/
│           ├── export.rs              (new)
│           └── import.rs              (new)
```

## Estimated Timeline

| Phase | Task | Hours |
|-------|------|-------|
| 1 | Menu entries | 2 |
| 2 | Export dialog (backend + UI) | 3-4 |
| 3 | Import dialog (backend + UI) | 4-5 |
| 4 | Integration & testing | 2-3 |
| **Total** | | **11-14 hours** |

## Risk Assessment

**High Risk:**
- Native UI implementation complexity (especially Windows)
- File dialog integration across platforms
- Clipboard operations across platforms

**Medium Risk:**
- Import validation logic
- Error handling edge cases
- Cross-platform testing

**Low Risk:**
- Menu entry creation (well-established pattern)
- CLI command integration (already working)

## Alternative Approaches

### Option A: Full Implementation (Recommended)
- Complete dialogs as described
- Best user experience
- 11-14 hours

### Option B: Simplified Version
- Use existing `textview` dialog for display
- Use native file dialogs only
- No dynamic validation
- 6-8 hours

### Option C: CLI-Only with Menu Shortcuts
- Menu entries open terminal with commands
- No GUI dialogs
- 2-3 hours

## Next Steps

1. **Get approval** on full implementation plan
2. **Start with Phase 1** (menu entries) - quick win
3. **Implement Phase 2** (export dialog) - simpler of the two
4. **Implement Phase 3** (import dialog) - more complex
5. **Test thoroughly** on macOS and Windows

## Questions for User

1. **Proceed with full implementation?** (Option A - 11-14 hours)
2. **Priority order:** Export first, then Import? Or both together?
3. **Linux support:** Basic functionality acceptable?
4. **Testing:** Can you test on both macOS and Windows as we go?

---

**Status:** Awaiting approval to proceed
**Created:** 2026-01-22
**Author:** Bob (AI Assistant)
