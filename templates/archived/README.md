# Archived Templates

These templates detect vulnerabilities that are **NOT related to flash loan attacks** and have been archived to improve scanning performance.

## Archived Templates (5)

1. **tx_origin_auth.yaml** - Phishing attacks using tx.origin
2. **signature_unchecked.yaml** - Signature verification issues
3. **integer_overflow_legacy.yaml** - Integer overflow in Solidity <0.8
4. **honeypot_detection.yaml** - Scam/honeypot detection
5. **cross_chain_gas_grief.yaml** - Cross-chain griefing attacks

## Why Archived?

These vulnerabilities:
- Cannot be exploited or amplified using flash loans
- Are independent attack vectors
- Slow down scanning without adding flash loan detection value

## How to Re-enable

To use these templates again:

```bash
# Move back to active templates
mv templates/archived/*.yaml templates/

# Or scan with archived templates specifically
scpf scan --templates templates/archived/
```

## Active Templates

All remaining templates in `templates/` directory (19 templates) are flash loan related and provide faster, focused scanning for flash loan vulnerabilities.
