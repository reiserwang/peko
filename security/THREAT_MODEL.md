# Threat Model: Peko

**Application**: Peko - Desktop web wrapper  
**Type**: Tauri v2 Desktop Application  
**Date**: 2026-01-08

---

## System Overview

```
┌─────────────────────────────────────────────────────┐
│                   Peko Desktop App                   │
├─────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────────────────┐   │
│  │  Frontend   │◄──►│     Tauri Runtime        │   │
│  │  (WebView)  │    │  (Rust IPC Commands)     │   │
│  └─────────────┘    └──────────────────────────┘   │
│         │                      │                    │
│         ▼                      ▼                    │
│  ┌─────────────┐    ┌──────────────────────────┐   │
│  │   Iframe    │    │     System Shell         │   │
│  │  (Sandbox)  │    │   (URL open, file)       │   │
│  └─────────────┘    └──────────────────────────┘   │
└─────────────────────────────────────────────────────┘
         │                      │
         ▼                      ▼
   ┌───────────┐        ┌───────────────┐
   │  Internet │        │  Local System │
   │ (Websites)│        │    (Files)    │
   └───────────┘        └───────────────┘
```

---

## Trust Boundaries

| Zone | Trust Level | Components |
|------|-------------|------------|
| **Trusted** | High | Rust backend, Tauri runtime |
| **Semi-Trusted** | Medium | Frontend JS, Bundled assets |
| **Untrusted** | Low | External websites, User input |

---

## STRIDE Analysis

### 1. Spoofing
| Asset | Threat | Likelihood | Impact | Mitigation |
|-------|--------|------------|--------|------------|
| App identity | Malicious app clone | Low | Medium | Code signing |
| Website content | Phishing in iframe | Medium | High | URL validation |

### 2. Tampering
| Asset | Threat | Likelihood | Impact | Mitigation |
|-------|--------|------------|--------|------------|
| App bundle | Modified binary | Low | Critical | Code signing, notarization |
| Frontend code | XSS injection | Medium | High | Enable CSP |

### 3. Repudiation
| Asset | Threat | Likelihood | Impact | Mitigation |
|-------|--------|------------|--------|------------|
| User actions | No audit trail | Medium | Low | Add logging |

### 4. Information Disclosure
| Asset | Threat | Likelihood | Impact | Mitigation |
|-------|--------|------------|--------|------------|
| Local files | file:// URL access | Medium | High | Block file:// scheme |
| Browsing history | Memory inspection | Low | Low | N/A |

### 5. Denial of Service
| Asset | Threat | Likelihood | Impact | Mitigation |
|-------|--------|------------|--------|------------|
| App resources | Memory exhaustion | Low | Low | OS protections |
| Infinite windows | Window spam | Medium | Medium | Rate limit window creation |

### 6. Elevation of Privilege
| Asset | Threat | Likelihood | Impact | Mitigation |
|-------|--------|------------|--------|------------|
| Shell access | Arbitrary command exec | Low | Critical | Strict capability restrictions |
| File system | Local file access via file:// | Medium | High | URL scheme validation |

---

## Attack Vectors

### AV-1: Malicious URL Injection
**Path**: User input → `navigate_to_url()` → WebviewWindow
**Risk**: javascript: or file:// URLs could execute code or access local files
**Status**: ⚠️ UNMITIGATED

### AV-2: XSS via External Content
**Path**: External website → iframe → DOM manipulation
**Risk**: If CSP is disabled, injected scripts could access Tauri APIs
**Status**: ⚠️ PARTIALLY MITIGATED (iframe sandboxed, but CSP disabled)

### AV-3: Dependency Vulnerabilities
**Path**: npm/cargo dependencies → bundled in app
**Risk**: Known CVEs in dependencies
**Status**: ✅ MITIGATED (npm audit clean, cargo audit recommended)

---

## Recommendations Summary

| Priority | Recommendation |
|----------|----------------|
| P0 | Enable CSP policy |
| P1 | Add URL scheme validation (http/https only) |
| P1 | Install cargo-audit for Rust vulnerability scanning |
| P2 | Add rate limiting for window creation |
| P3 | Implement audit logging |
