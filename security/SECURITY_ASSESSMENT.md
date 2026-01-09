# Security Assessment Report: Peko

**Date**: 2026-01-08  
**Reviewer**: Security Agent  
**Scope**: Full Tauri desktop application

---

## 📦 SBOM Summary

| Category | Count |
|----------|-------|
| **NPM Dependencies** | 17 (including 6 dev) |
| **Rust Crates** | 456 (via Cargo) |
| **Known NPM Vulnerabilities** | 0 |
| **Cargo Audit** | Not installed (recommended) |

---

## 🚨 Critical Issues (Must Fix)

### 1. [A05] CSP Disabled - Security Misconfiguration
**Location**: `src-tauri/tauri.conf.json:30-31`

```json
"security": {
    "csp": null,
    "dangerousDisableAssetCspModification": true
}
```

**Risk**: Content Security Policy is completely disabled, allowing:
- Inline script execution
- External resource loading without restrictions
- XSS attack vectors if malicious content is injected

**Recommendation**: Enable CSP with appropriate directives:
```json
"security": {
    "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
}
```

---

## ⚠️ Potential Risks (Should Fix)

### 2. [A04] URL Navigation Without Validation
**Location**: `src-tauri/src/lib.rs:5-21`

```rust
fn navigate_to_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed_url))
```

**Risk**: 
- No URL scheme validation (allows `file://`, `javascript:`, etc.)
- No domain allowlist
- Potential for SSRF or local file access

**Recommendation**: Add URL validation:
```rust
fn validate_url(url: &str) -> Result<url::Url, String> {
    let parsed = url.parse::<url::Url>().map_err(|e| format!("Invalid URL: {}", e))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        _ => Err("Only HTTP/HTTPS URLs are allowed".into())
    }
}
```

### 3. [A01] Overly Permissive Capabilities
**Location**: `src-tauri/capabilities/default.json`

**Risk**: The `shell:allow-open` permission allows opening arbitrary URLs/files via system shell.

**Recommendation**: Consider restricting to specific protocols:
```json
"shell:allow-open": {
    "open": {
        "allowedSchemes": ["https", "mailto"]
    }
}
```

### 4. [A09] No Audit Logging
**Risk**: No logging of navigation events or Tauri command invocations for forensic analysis.

**Recommendation**: Add logging for security-relevant events.

---

## 🟢 Positive Findings

| Check | Status |
|-------|--------|
| Hardcoded secrets | ✅ None found |
| XSS vectors (eval, innerHTML) | ✅ None found |
| NPM vulnerabilities | ✅ 0 found |
| Iframe sandboxing | ✅ Properly configured |
| CORS handling | ✅ Uses sandboxed iframe |

---

## 🛡️ Hardening Recommendations

| Priority | Action |
|----------|--------|
| **P0** | Enable CSP in `tauri.conf.json` |
| **P1** | Add URL scheme validation in `lib.rs` |
| **P1** | Install and run `cargo audit` for Rust CVE scanning |
| **P2** | Restrict shell:allow-open to specific schemes |
| **P3** | Add security-event logging |

---

## 🔍 Threat Model (STRIDE)

| Threat | Applicable | Mitigation |
|--------|------------|------------|
| **Spoofing** | Low | N/A - local desktop app |
| **Tampering** | Medium | Enable CSP, code signing |
| **Repudiation** | Medium | Add audit logging |
| **Info Disclosure** | Low | No sensitive data stored |
| **DoS** | Low | OS-level protection |
| **Elevation of Privilege** | Medium | Validate URL schemes |

---

## Files Reviewed

- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`
- `src/main.js`
- `src/index.html`
- `package.json`

---

## ✅ Mitigations Applied (2026-01-08)

| Issue | Fix Applied |
|-------|-------------|
| **P0: CSP Disabled** | Enabled CSP with `default-src 'self'` and strict directives |
| **P1: URL Validation** | Added `validate_url()` function blocking `file://`, `javascript:`, and other dangerous schemes |
| **P2: Shell Capabilities** | Restricted `shell:allow-open` to `https://`, `http://`, and `mailto:` only |
| **P3: Audit Logging** | Added `env_logger` with startup and navigation logging |

### Verification

```bash
cargo check  # ✅ 0 errors, 0 warnings
```
