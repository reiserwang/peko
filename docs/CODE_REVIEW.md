# Code Review Report: Peko Application

**Date**: 2026-01-13  
**Reviewer**: Code Reviewer Agent (Orchestrator delegated)  
**Scope**: Full codebase review

---

## Executive Summary

| Metric | Result |
|:-------|:-------|
| **Clippy** | ✅ 0 warnings |
| **Build** | ✅ Pass |
| **Tests** | ✅ 9/9 pass |
| **Lines Reviewed** | 1,129 |

**Overall Rating**: ⭐⭐⭐⭐ Good

---

## Files Reviewed

| File | Lines | Purpose |
|:-----|:------|:--------|
| `src-tauri/src/lib.rs` | 860 | Rust backend |
| `src/main.js` | 203 | Settings UI |
| `src/notes.js` | 66 | Notes panel |

---

## 🟢 Strengths

### Code Organization
- ✅ Clean separation of concerns (Tauri commands are well-isolated)
- ✅ Consistent error handling with `Result<T, String>`
- ✅ Good use of serde defaults for backward compatibility

### Security
- ✅ URL protocol enforcement in `saveAndClose()` (forces https://)
- ✅ HTML escaping with `escapeHtml()` function
- ✅ CSP enabled in Tauri config

### Testing
- ✅ 9 unit tests covering core structs and serialization

---

## 🟡 Suggestions (P2)

### 1. Mutex Lock Unwrap Pattern
**Location**: Multiple places in `lib.rs`

```rust
// Current (panics on poisoned mutex)
let settings = state.0.lock().unwrap();

// Suggested (graceful handling)
let settings = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
```

**Risk**: Low — mutex poisoning rare, but panics in production are undesirable.

---

### 2. Magic Number Constants
**Location**: `lib.rs:102`, `lib.rs:295`

```rust
if websites.len() > 5 { ... }  // Magic number
let sidebar_width: u32 = 350;   // Magic number
```

**Suggestion**: Extract to named constants:
```rust
const MAX_WEBSITES: usize = 5;
const SIDEBAR_WIDTH: u32 = 350;
```

---

### 3. Notes Mode Could Use Enum
**Location**: `lib.rs:30`, `lib.rs:283-288`

```rust
// Current: String-based mode
pub notes_mode: String,  // "hidden", "sidebar", "window"

// Suggested: Type-safe enum
#[derive(Serialize, Deserialize)]
enum NotesMode { Hidden, Sidebar, Window }
```

**Benefit**: Compile-time safety, exhaustive pattern matching.

---

### 4. JavaScript: Consider Defensive Checks
**Location**: `main.js:65-67`

```javascript
// Current (could fail if index out of bounds)
const index = parseInt(e.target.dataset.index);
websites[index][field] = e.target.value;

// Suggested
if (index >= 0 && index < websites.length) {
  websites[index][field] = e.target.value;
}
```

---

### 5. Error Handling in Notes Preview
**Location**: `notes.js:60-64`

```javascript
// Current: Silently falls back if marked is undefined
if (typeof marked !== 'undefined') {
  preview.innerHTML = marked.parse(editor.value || '*No notes yet*');
}
```

**Note**: This is acceptable but could log a warning for debugging.

---

## 🔴 Critical Issues

**None found.** ✅

---

## Metrics

| Category | Count |
|:---------|:------|
| Critical | 0 |
| Warnings | 0 (Clippy) |
| Suggestions | 5 |
| Unit Tests | 9 |

---

## Recommendations Summary

| Priority | Action |
|:---------|:-------|
| P2 | Extract magic numbers to constants |
| P2 | Consider enum for `notes_mode` |
| P3 | Add defensive array bounds checks in JS |
| P3 | Graceful mutex error handling |

---

## Verification Commands Used

```bash
cargo clippy -- -W clippy::all  # ✅ 0 warnings
cargo test                       # ✅ 9 tests pass
cargo build                      # ✅ Success
```
