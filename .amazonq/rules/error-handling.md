# Error Handling Rules

**CRITICAL**: Never use empty catch blocks or silently suppress errors.

## ❌ FORBIDDEN:
- Empty catch blocks
- Silent error suppression
- Underscore-prefixed unused errors

## ✅ REQUIRED:
- Always handle errors explicitly
- Re-throw unexpected errors
- Log critical errors with context
