# Phase 2: Export Dialog Implementation Plan

## Status: ✅ Phase 1 Complete - Menu Entries Working

## Architecture Decision

After analyzing the modulo codebase, I've identified two approaches:

### Approach A: Form-Based (Recommended - Faster)
Use existing form infrastructure:
- Checkboxes for scope selection (Config, Matches, Packages)
- Submit button triggers export
- Show result in textview dialog
- **Pros:** Reuses existing code, faster to implement
- **Cons:** Less flexible UI, two-step process

### Approach B: Custom Dialog (More Complex)
Create new custom dialog type:
- Integrated UI with live preview
- Copy/Save buttons
- **Pros:** Better UX, single dialog
- **Cons:** Requires C++/Objective-C code, longer implementation

## Recommended Implementation: Approach A

### Step 1: Create Export Handler in Engine
**File:** `espanso-engine/src/event/mod.rs`
- Add `ExportConfig` event type

**File:** `espanso-engine/src/process/middleware/context_menu.rs`
- Update `CONTEXT_ITEM_EXPORT_CONFIG` handler to dispatch `ExportConfig` event

### Step 2: Create Export Executor
**File:** `espanso/src/cli/worker/engine/dispatch/executor/export.rs`
- Handle `ExportConfig` event
- Launch form dialog for scope selection
- Execute export command with selected scopes
- Show result in textview dialog

### Step 3: Wire Up in Engine
**File:** `espanso/src/cli/worker/engine/mod.rs`
- Register export executor

## Implementation Steps

1. Add `ExportConfig` event type
2. Create export executor
3. Create form schema for scope selection
4. Execute export command
5. Display result in textview
6. Test on macOS
7. Test on Windows

## Estimated Time: 2-3 hours

## Next Steps After Export
- Implement Import dialog (similar approach)
- Add validation for import
- Add confirmation dialogs
- Document Linux implementation

---

**Ready to proceed with Approach A?**
