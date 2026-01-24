# Clippy Fehler Behebungsplan

## Identifizierte Fehler

### 1. clippy::ref_as_ptr Fehler
**Datei:** `espanso-modulo/src/sys/form/mod.rs:347`
**Problem:** Verwendung von `std::ptr::from_ref` löst Clippy-Warnung aus

**Aktueller Code (Zeile 347):**
```rust
fn as_ptr(&self) -> *const c_void {
    std::ptr::from_ref::<bool>(self) as *const c_void
}
```

**Lösung:** Verwende `as *const _` statt `std::ptr::from_ref`
```rust
fn as_ptr(&self) -> *const c_void {
    self as *const bool as *const c_void
}
```

**Auch betroffen (Zeile 353):**
```rust
fn as_ptr(&self) -> *const c_void {
    std::ptr::from_ref::<RowMetadata>(&(*self.interop)) as *const c_void
}
```

**Lösung:**
```rust
fn as_ptr(&self) -> *const c_void {
    &(*self.interop) as *const RowMetadata as *const c_void
}
```

### 2. clippy::map_unwrap_or Fehler
**Datei:** `espanso-modulo/src/sys/import_dialog/mod.rs:44`
**Problem:** Verwendung von `.map().unwrap_or()` statt `.map_or()`

**Aktueller Code (Zeilen 44-46):**
```rust
window_icon_path: icon_path_c
    .as_ref()
    .map(|c| c.as_ptr())
    .unwrap_or(std::ptr::null()),
```

**Lösung:** Verwende `map_or` direkt
```rust
window_icon_path: icon_path_c
    .as_ref()
    .map_or(std::ptr::null(), |c| c.as_ptr()),
```

## Umsetzungsschritte

1. ✅ Fehler identifiziert und analysiert
2. ⏳ In 'code' oder 'advanced' Modus wechseln
3. ⏳ Datei `espanso-modulo/src/sys/form/mod.rs` bearbeiten:
   - Zeile 347: `std::ptr::from_ref::<bool>(self)` → `self as *const bool`
   - Zeile 353: `std::ptr::from_ref::<RowMetadata>(&(*self.interop))` → `&(*self.interop) as *const RowMetadata`
4. ⏳ Datei `espanso-modulo/src/sys/import_dialog/mod.rs` bearbeiten:
   - Zeilen 44-46: `.map(|c| c.as_ptr()).unwrap_or(std::ptr::null())` → `.map_or(std::ptr::null(), |c| c.as_ptr())`
5. ⏳ Änderungen committen
6. ⏳ Änderungen pushen

## Commit-Nachricht
```
fix: resolve clippy warnings for ref_as_ptr and map_unwrap_or

- Replace std::ptr::from_ref with direct pointer casts in form/mod.rs
- Replace map().unwrap_or() with map_or() in import_dialog/mod.rs

Fixes clippy errors in CI build #21300914228
```
