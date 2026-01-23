# Offline Import/Export Feature - Fix Implementation Plan

## Executive Summary

This document outlines a comprehensive plan to fix critical and minor issues in the offline import/export feature (`espanso/src/cli/offline.rs`). The feature currently works well for interactive use but has critical failures in non-interactive environments.

**Current Status:** Production-ready for interactive use, requires fixes for automation
**Target:** Full cross-platform compatibility in all environments

---

## Issue Priority Matrix

| Priority | Issue | Impact | Effort | Risk |
|----------|-------|--------|--------|------|
| 🔴 P0 | Unix TTY confirmation failure | High | Low | Low |
| 🔴 P0 | Windows console service mode | High | Medium | Medium |
| 🟡 P1 | Temp file cleanup on error | Medium | Low | Low |
| 🟡 P1 | Error message context | Medium | Low | Low |
| 🟢 P2 | Documentation (rustdoc) | Low | Medium | Low |
| 🟢 P2 | Code duplication | Low | Medium | Low |
| 🟢 P3 | Integration tests | Low | High | Low |
| 🟢 P3 | Optional newline flag | Low | Low | Low |

---

## Phase 1: Critical Fixes (P0)

### 1.1 Fix Unix Confirmation Input (Lines 428-456)

**Problem:** Direct `/dev/tty` access fails in non-interactive environments

**Current Code:**
```rust
#[cfg(unix)]
fn read_confirmation_byte_unix() -> Result<u8> {
    let mut file = fs::File::open("/dev/tty")?;
    // ... rest of implementation
}
```

**Solution:**
```rust
#[cfg(unix)]
fn read_confirmation_byte_unix() -> Result<u8> {
    use std::mem;
    use std::os::unix::io::AsRawFd;

    // Try /dev/tty first, fallback to stdin
    let mut file = fs::File::open("/dev/tty")
        .or_else(|_| {
            // If no TTY available, try stdin
            // This allows piped input to work
            std::io::stdin().lock();
            fs::File::open("/dev/stdin")
        })?;
    
    let fd = file.as_raw_fd();
    // ... rest remains the same
}
```

**Testing:**
- Test in interactive terminal: `espanso import < data.txt`
- Test in non-interactive: `echo "y" | espanso import < data.txt`
- Test in cron job simulation
- Test in Docker without TTY
- Test with `--yes` flag (should bypass this code)

**Acceptance Criteria:**
- ✅ Works in interactive terminal
- ✅ Works with piped input
- ✅ Works in non-TTY environments
- ✅ Proper error message if both fail

---

### 1.2 Fix Windows Console Service Mode (Lines 467-507)

**Problem:** `CONIN$` access fails when running as Windows service

**Current Code:**
```rust
#[cfg(windows)]
fn read_confirmation_byte_windows() -> Result<u8> {
    let file = fs::File::open("CONIN$").or_else(|_| {
        let _ = crate::util::attach_console();
        fs::File::open("CONIN$")
    })?;
    // ... rest
}
```

**Solution:**
```rust
#[cfg(windows)]
fn read_confirmation_byte_windows() -> Result<u8> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{
        GetConsoleMode, ReadConsoleA, SetConsoleMode, 
        ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
    };

    // Try CONIN$ first
    let file = fs::File::open("CONIN$")
        .or_else(|_| {
            // Try to attach console
            let _ = crate::util::attach_console();
            fs::File::open("CONIN$")
        })
        .or_else(|_| {
            // Last resort: try stdin
            // This won't work for raw input but better than crashing
            std::io::stdin();
            bail!("No console available. Use --yes flag for non-interactive mode.")
        })?;
    
    // ... rest remains the same
}
```

**Alternative Approach:**
Consider detecting service mode early and requiring `--yes` flag:

```rust
fn confirm_import(skip_confirmation: bool) -> Result<()> {
    if skip_confirmation {
        return Ok(());
    }

    // Detect if running in non-interactive mode
    #[cfg(windows)]
    if !is_console_available() {
        bail!("Running in non-interactive mode. Use --yes flag to skip confirmation.");
    }

    // ... rest of confirmation logic
}

#[cfg(windows)]
fn is_console_available() -> bool {
    use windows::Win32::System::Console::GetConsoleWindow;
    unsafe { !GetConsoleWindow().is_invalid() }
}
```

**Testing:**
- Test in interactive CMD/PowerShell
- Test as Windows service
- Test with `--yes` flag
- Test in scheduled task

**Acceptance Criteria:**
- ✅ Works in interactive console
- ✅ Fails gracefully with helpful message in service mode
- ✅ `--yes` flag bypasses all checks
- ✅ Clear error message guides user to solution

---

## Phase 2: High Priority Fixes (P1)

### 2.1 Add Temp File Cleanup (Lines 644-656)

**Problem:** Temp files left behind on write failure

**Current Code:**
```rust
fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = temp_path_for(path);
    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
    }

    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}
```

**Solution:**
```rust
fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = temp_path_for(path);
    
    // Ensure cleanup on error
    struct TempFileGuard<'a> {
        path: &'a Path,
    }
    
    impl<'a> Drop for TempFileGuard<'a> {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.path);
        }
    }
    
    let _guard = TempFileGuard { path: &tmp_path };
    
    {
        let mut file = fs::File::create(&tmp_path)
            .context("failed to create temporary file")?;
        file.write_all(data)
            .context("failed to write to temporary file")?;
        file.sync_all()
            .context("failed to sync temporary file")?;
    }

    if path.exists() {
        fs::remove_file(path)
            .context("failed to remove existing file")?;
    }
    
    fs::rename(&tmp_path, path)
        .context("failed to rename temporary file")?;
    
    // Prevent cleanup on success
    std::mem::forget(_guard);
    
    Ok(())
}
```

**Testing:**
- Test normal write success
- Test write failure (disk full simulation)
- Test permission errors
- Verify temp files are cleaned up

**Acceptance Criteria:**
- ✅ Temp files cleaned up on any error
- ✅ No cleanup on success
- ✅ Better error messages with context

---

### 2.2 Improve Error Messages with File Context

**Problem:** Generic errors don't indicate which file failed

**Locations to Fix:**
1. `import_payload_from_stdin()` - Line 520-560
2. `append_dir_to_archive()` - Line 340-380
3. `write_atomic()` - Already improved above

**Solution Pattern:**
```rust
// Before:
entry.read_to_end(&mut data)?;

// After:
entry.read_to_end(&mut data)
    .with_context(|| format!("failed to read archive entry: {}", entry_path.display()))?;

// Before:
write_atomic(&target_path, &data)?;

// After:
write_atomic(&target_path, &data)
    .with_context(|| format!("failed to write file: {}", target_path.display()))?;
```

**Implementation Steps:**
1. Audit all `?` operators in import/export functions
2. Add `.with_context()` with file path information
3. Ensure error messages are actionable

**Testing:**
- Trigger various error conditions
- Verify error messages include file paths
- Check error message clarity

**Acceptance Criteria:**
- ✅ All file operations have contextual errors
- ✅ Error messages include file paths
- ✅ Users can identify which file caused the issue

---

## Phase 3: Documentation (P2)

### 3.1 Add Rustdoc Comments

**Locations:**
- Module level (top of file)
- `new_export()` function
- `new_import()` function
- `export_payload_to_stdout()`
- `import_payload_from_stdin()`
- `WhitespaceFilteringReader`
- `WrapWriter`

**Template:**
```rust
//! Offline import/export functionality for Espanso configuration.
//!
//! This module provides commands to export Espanso configuration, matches,
//! and packages as a base64-encoded payload for offline transfer between
//! machines. The payload is compressed using gzip and encoded in base64
//! for easy copy-paste or file transfer.
//!
//! # Usage
//!
//! Export all data:
//! ```bash
//! espanso export > backup.txt
//! ```
//!
//! Export specific scopes:
//! ```bash
//! espanso export --scope config,matches > backup.txt
//! ```
//!
//! Import data (interactive):
//! ```bash
//! espanso import < backup.txt
//! ```
//!
//! Import data (non-interactive):
//! ```bash
//! espanso import --yes < backup.txt
//! ```
//!
//! # Security
//!
//! - Path traversal protection via `sanitize_relative_path()`
//! - Root boundary enforcement via `ensure_inside_root()`
//! - Atomic file writes to prevent corruption
//! - User confirmation before destructive operations
//!
//! # Platform Support
//!
//! - macOS: Full support
//! - Windows: Full support
//! - Linux: Full support
//!
//! Note: Interactive confirmation requires a TTY. Use `--yes` flag
//! for non-interactive environments (CI/CD, services, cron jobs).

/// Creates the CLI module for the export command.
///
/// Exports Espanso configuration data as a base64-encoded payload.
/// The output is written to stdout and can be redirected to a file.
///
/// # Examples
///
/// ```bash
/// espanso export > backup.txt
/// espanso export --scope config > config-only.txt
/// espanso export --wrap 76 > wrapped-backup.txt
/// ```
pub fn new_export() -> CliModule {
    // ...
}
```

**Acceptance Criteria:**
- ✅ Module-level documentation complete
- ✅ All public functions documented
- ✅ Examples provided for common use cases
- ✅ Platform limitations documented
- ✅ Security features documented

---

### 3.2 Add Inline Comments for Complex Logic

**Locations:**
1. `matches_skip_packages` logic (Line 308)
2. `prepare_import_targets()` package handling (Line 580)
3. `WhitespaceFilteringReader` implementation (Line 48)
4. Platform-specific confirmation code

**Example:**
```rust
// Check if packages directory is nested inside matches directory.
// This happens when using the default configuration where packages
// are stored at config/match/packages. In this case, we need to
// skip the packages directory when exporting matches to avoid
// duplication, since packages will be exported separately.
let matches_skip_packages =
    matches_dir.is_dir() 
    && packages_dir.is_dir() 
    && packages_dir.starts_with(&matches_dir);
```

---

## Phase 4: Code Quality Improvements (P2)

### 4.1 Extract Common Archive Building Logic

**Problem:** Duplication between `export_payload_to_stdout()` and test helper `export_to_vec()`

**Solution:**
```rust
/// Core archive building logic shared between export functions.
fn build_archive<W: Write>(
    writer: W,
    paths: &Paths,
    selection: ScopeSelection,
) -> Result<()> {
    let mut gzip = GzEncoder::new(writer, Compression::default());
    let mut builder = Builder::new(&mut gzip);

    let matches_dir = paths.config.join("match");
    let packages_dir = paths.packages.clone();
    let matches_skip_packages =
        matches_dir.is_dir() 
        && packages_dir.is_dir() 
        && packages_dir.starts_with(&matches_dir);

    if selection.config {
        let config_dir = paths.config.join("config");
        append_dir_to_archive(&mut builder, &config_dir, Path::new("config"), None)?;
    }
    if selection.matches {
        let skip_dir = if matches_skip_packages {
            Some(packages_dir.as_path())
        } else {
            None
        };
        append_dir_to_archive(
            &mut builder,
            &paths.config.join("match"),
            Path::new("matches"),
            skip_dir,
        )?;
    }
    if selection.packages {
        append_dir_to_archive(&mut builder, &packages_dir, Path::new("packages"), None)?;
    }

    builder.finish()?;
    drop(builder);
    gzip.finish()?;
    Ok(())
}

fn export_payload_to_stdout(
    paths: &Paths,
    selection: ScopeSelection,
    wrap: Option<usize>,
) -> Result<()> {
    let stdout = io::stdout();
    let handle = stdout.lock();
    let writer: Box<dyn Write + '_> = if let Some(width) = wrap {
        Box::new(WrapWriter::new(handle, width))
    } else {
        Box::new(handle)
    };
    let encoder = EncoderWriter::new(writer, &STANDARD);
    
    build_archive(encoder, paths, selection)?;
    
    let mut handle = encoder.finish()?;
    handle.write_all(b"\n")?;
    Ok(())
}
```

**Benefits:**
- Reduces duplication
- Easier to maintain
- Consistent behavior between production and tests

---

## Phase 5: Testing (P3)

### 5.1 Add Integration Tests for Non-Interactive Mode

**Test File:** `espanso/tests/offline_integration_tests.rs`

```rust
#[test]
fn test_export_import_non_interactive() {
    let temp = TempDir::new("espanso-test").unwrap();
    
    // Setup test data
    let paths = create_test_paths(&temp);
    create_test_files(&paths);
    
    // Export
    let export_output = Command::new(espanso_binary())
        .args(&["export"])
        .output()
        .expect("export failed");
    
    assert!(export_output.status.success());
    
    // Import with --yes flag
    let import_output = Command::new(espanso_binary())
        .args(&["import", "--yes"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("import spawn failed")
        .stdin
        .unwrap()
        .write_all(&export_output.stdout)
        .expect("write to stdin failed");
    
    // Verify imported data
    verify_imported_files(&paths);
}

#[test]
fn test_import_without_tty() {
    // Simulate non-TTY environment
    // This should fail without --yes flag
    let result = Command::new(espanso_binary())
        .args(&["import"])
        .env("TERM", "dumb")
        .stdin(Stdio::piped())
        .output();
    
    assert!(!result.unwrap().status.success());
}

#[test]
fn test_import_with_yes_flag_no_tty() {
    // Should succeed with --yes even without TTY
    let result = Command::new(espanso_binary())
        .args(&["import", "--yes"])
        .env("TERM", "dumb")
        .stdin(Stdio::piped())
        .output();
    
    assert!(result.unwrap().status.success());
}
```

**Test Coverage:**
- ✅ Export in various environments
- ✅ Import with `--yes` flag
- ✅ Import without TTY (should fail gracefully)
- ✅ Scope selection (config, matches, packages)
- ✅ Line break conversion
- ✅ Wrapped output
- ✅ Error handling

---

### 5.2 Add Unit Tests for TTY-less Scenarios

**Add to existing test module:**

```rust
#[test]
fn test_confirmation_with_yes_flag() {
    let result = confirm_import(true);
    assert!(result.is_ok());
}

#[test]
fn test_confirmation_without_yes_flag_requires_input() {
    // This test documents that confirmation requires input
    // In real scenarios, this would block waiting for input
}

#[test]
fn test_whitespace_filtering_comprehensive() {
    let input = b"a\nb\r\nc\td \n\r\t e";
    let mut reader = WhitespaceFilteringReader::new(&input[..]);
    let mut output = String::new();
    reader.read_to_string(&mut output).unwrap();
    assert_eq!(output, "abcde");
}

#[test]
fn test_wrap_writer_at_boundary() {
    let mut output = Vec::new();
    {
        let mut writer = WrapWriter::new(&mut output, 4);
        writer.write_all(b"abcdefgh").unwrap();
    }
    assert_eq!(output, b"abcd\nefgh");
}
```

---

## Phase 6: Optional Enhancements (P3)

### 6.1 Optional Final Newline Flag

**Add flag to export command:**

```rust
.subcommand(
  SubCommand::with_name("export")
    .about("Export Espanso data as a base64 payload for offline transfer.")
    .arg(
      Arg::with_name("scope")
        .long("scope")
        .takes_value(true)
        .help("Comma-separated list of scopes: config,matches,packages (default all)"),
    )
    .arg(
      Arg::with_name("wrap")
        .long("wrap")
        .takes_value(true)
        .help("Wrap base64 output at the given column width"),
    )
    .arg(
      Arg::with_name("no-newline")
        .long("no-newline")
        .takes_value(false)
        .help("Don't add a final newline to the output"),
    ),
)
```

**Implementation:**
```rust
fn export_main(args: CliModuleArgs) -> i32 {
    // ... existing code ...
    
    let no_newline = sub_args.is_present("no-newline");
    
    if let Err(err) = export_payload_to_stdout(&paths, scope_selection, wrap, no_newline) {
        error_eprintln!("unable to export: {err}");
        return 1;
    }
    
    0
}

fn export_payload_to_stdout(
    paths: &Paths,
    selection: ScopeSelection,
    wrap: Option<usize>,
    no_newline: bool,
) -> Result<()> {
    // ... existing code ...
    
    if !no_newline {
        handle.write_all(b"\n")?;
    }
    Ok(())
}
```

---

### 6.2 Add Cleanup Logic for Orphaned Temp Files

**Add cleanup function:**

```rust
/// Cleans up orphaned temporary files from failed imports.
///
/// Searches for files matching the pattern `*.espanso_import_tmp_*`
/// that are older than 24 hours and removes them.
fn cleanup_orphaned_temp_files(paths: &Paths) -> Result<()> {
    let now = SystemTime::now();
    let cutoff = now - Duration::from_secs(24 * 60 * 60);
    
    for scope in [Scope::Config, Scope::Matches, Scope::Packages] {
        let root = scope_root(paths, scope);
        if !root.exists() {
            continue;
        }
        
        for entry in WalkDir::new(&root).follow_links(false) {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains(".espanso_import_tmp_") {
                    if let Ok(metadata) = fs::metadata(path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified < cutoff {
                                let _ = fs::remove_file(path);
                                info!("Cleaned up orphaned temp file: {}", path.display());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
```

**Call during import:**
```rust
fn import_main(args: CliModuleArgs) -> i32 {
    let paths = args.paths.expect("missing paths argument");
    
    // Clean up old temp files before starting
    let _ = cleanup_orphaned_temp_files(&paths);
    
    // ... rest of import logic ...
}
```

---

## Implementation Timeline

### Sprint 1 (Week 1): Critical Fixes
- **Day 1-2:** Fix Unix TTY confirmation with fallback
- **Day 3-4:** Fix Windows console service mode
- **Day 5:** Testing and validation

### Sprint 2 (Week 2): High Priority
- **Day 1-2:** Add temp file cleanup
- **Day 3-4:** Improve error messages
- **Day 5:** Testing and validation

### Sprint 3 (Week 3): Documentation
- **Day 1-3:** Add rustdoc comments
- **Day 4-5:** Add inline comments and review

### Sprint 4 (Week 4): Code Quality & Testing
- **Day 1-2:** Extract common logic
- **Day 3-5:** Add integration tests

### Sprint 5 (Week 5): Optional Enhancements
- **Day 1-2:** Optional newline flag
- **Day 3-4:** Temp file cleanup logic
- **Day 5:** Final testing and documentation

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing functionality | Low | High | Comprehensive testing, backward compatibility |
| Platform-specific bugs | Medium | Medium | Test on all platforms before merge |
| Performance regression | Low | Low | Benchmark before/after |
| User confusion with new flags | Low | Low | Clear documentation and examples |

---

## Success Criteria

### Must Have (P0-P1):
- ✅ Import works in non-interactive environments with `--yes` flag
- ✅ Graceful error messages when TTY not available
- ✅ No temp files left behind on errors
- ✅ Error messages include file context
- ✅ All existing tests pass
- ✅ Works on macOS, Windows, and Linux

### Should Have (P2):
- ✅ Comprehensive rustdoc documentation
- ✅ Reduced code duplication
- ✅ Clear inline comments for complex logic

### Nice to Have (P3):
- ✅ Integration tests for all scenarios
- ✅ Optional newline flag
- ✅ Automatic temp file cleanup

---

## Testing Strategy

### Unit Tests:
- Confirmation logic with/without TTY
- Whitespace filtering edge cases
- Path sanitization
- Scope selection parsing
- Temp file cleanup

### Integration Tests:
- Full export/import cycle
- Non-interactive mode
- Scope selection
- Line break conversion
- Error scenarios

### Manual Testing:
- Interactive terminal (macOS, Windows, Linux)
- Non-interactive (cron, service, Docker)
- SSH without TTY
- CI/CD pipeline
- Various error conditions

### Platform Testing:
- macOS 12+ (Intel & Apple Silicon)
- Windows 10/11
- Linux (Ubuntu, Fedora, Arch)

---

## Rollback Plan

If critical issues are discovered:

1. **Immediate:** Revert to previous version
2. **Short-term:** Fix issues in development branch
3. **Long-term:** Add more comprehensive tests

**Rollback Triggers:**
- Data loss or corruption
- Complete failure on any platform
- Security vulnerability discovered

---

## Documentation Updates

### User Documentation:
- Update README with `--yes` flag usage
- Add troubleshooting section for non-interactive mode
- Document platform-specific limitations

### Developer Documentation:
- Update CONTRIBUTING.md with testing requirements
- Document architecture decisions
- Add examples for future contributors

---

## Conclusion

This plan provides a structured approach to fixing all identified issues in the offline import/export feature. By following this plan, we will achieve:

1. **Reliability:** Works in all environments (interactive and non-interactive)
2. **Robustness:** Proper error handling and cleanup
3. **Maintainability:** Well-documented and tested code
4. **User Experience:** Clear error messages and documentation

**Estimated Total Effort:** 3-5 weeks (1 developer)
**Risk Level:** Low (with proper testing)
**Impact:** High (enables automation and improves reliability)
