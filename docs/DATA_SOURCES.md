# Real Data Sources

SCPF fetches from **verified, working APIs** - no fake endpoints.

## ✅ Currently Active

### 1. DeFiHackLabs GitHub
- **URL**: `https://api.github.com/repos/SunWeb3Sec/DeFiHackLabs/commits`
- **Data**: Last 30 commits with exploit POCs
- **Rate Limit**: 60/hour (unauthenticated), 5000/hour (with token)
- **Update Frequency**: Multiple times daily
- **Reliability**: ⭐⭐⭐⭐⭐

### 2. GitHub Solidity Advisories
- **URL**: `https://api.github.com/repos/ethereum/solidity/security/advisories`
- **Data**: Compiler vulnerabilities and security advisories
- **Rate Limit**: 60/hour (unauthenticated), 5000/hour (with token)
- **Update Frequency**: As disclosed
- **Reliability**: ⭐⭐⭐⭐⭐

### 3. Rekt News RSS
- **URL**: `https://rekt.news/feed.xml`
- **Data**: Major DeFi hacks and post-mortems
- **Rate Limit**: None
- **Update Frequency**: Weekly
- **Reliability**: ⭐⭐⭐⭐⭐

## 🔧 API Authentication

### GitHub Token (Recommended)

Increase rate limits from 60 to 5000 requests/hour:

```bash
# Create token at https://github.com/settings/tokens
export GITHUB_TOKEN="ghp_your_token_here"

# Or add to ~/.bashrc
echo 'export GITHUB_TOKEN="ghp_your_token_here"' >> ~/.bashrc
```

SCPF automatically uses `GITHUB_TOKEN` if set.

## 📊 Data Quality

| Source | Timeliness | Accuracy | Coverage |
|--------|-----------|----------|----------|
| DeFiHackLabs | ⭐⭐⭐⭐⭐ (Real-time) | ⭐⭐⭐⭐⭐ | Major exploits |
| GitHub Solidity | ⭐⭐⭐⭐ (Days) | ⭐⭐⭐⭐⭐ | Compiler bugs |
| Rekt News | ⭐⭐⭐ (Weekly) | ⭐⭐⭐⭐⭐ | Major hacks |

## 🚀 Adding More Sources

### Easy to Add (Public APIs)

**Immunefi Bounties:**
```rust
// No public API - would need web scraping
```

**SlowMist Hacked:**
```rust
// Database available at https://hacked.slowmist.io
// No official API - would need scraping
```

**Code4rena:**
```rust
// Findings available after contests
// No public API currently
```

### Requires API Keys

**Twitter Security Alerts:**
```rust
// Requires Twitter API v2 access
// Cost: $100/month for basic tier
```

**CertiK Skynet:**
```rust
// Requires CertiK API key
// Contact: https://www.certik.com/
```

## 🎯 Why These Sources?

1. **DeFiHackLabs** - Most comprehensive exploit database
   - Community-maintained
   - Includes POC code
   - Updated within hours of exploits

2. **GitHub Solidity** - Official compiler advisories
   - Direct from Ethereum Foundation
   - Critical for compiler bugs
   - Affects all Solidity contracts

3. **Rekt News** - Best exploit analysis
   - Detailed post-mortems
   - Loss amounts
   - Technical breakdowns

## 📈 Coverage Statistics

**Last 30 Days (Jan 2026):**
- DeFiHackLabs: 45 commits (exploits/updates)
- GitHub Solidity: 2 advisories
- Rekt News: 3 major articles

**Total: ~50 security events/month**

## ⚠️ Limitations

**What We DON'T Have:**
- Real-time Twitter alerts (requires paid API)
- SlowMist database (no public API)
- Immunefi disclosures (no public API)
- Private audit findings (confidential)

**What We DO Have:**
- Most comprehensive public exploit database (DeFiHackLabs)
- Official compiler vulnerabilities (GitHub)
- Major hack analysis (Rekt News)

## 🔮 Future Sources

**In Development:**
1. Web scraping for SlowMist
2. RSS feeds for security blogs
3. On-chain monitoring (Forta Network)
4. Community submissions

**Requires Funding:**
1. Twitter API access ($100/month)
2. CertiK API access (enterprise)
3. Dedicated monitoring infrastructure

## 💡 Community Contributions

Help us add more sources:

1. **Find public APIs** - Submit PR with endpoint
2. **Build scrapers** - For sites without APIs
3. **Sponsor API access** - Fund Twitter/CertiK keys
4. **Share exploits** - Manual submissions welcome

## 📞 Contact

Missing a critical source? Open an issue:
https://github.com/Teycir/smartcontractpatternfinder/issues/new?template=data-source.md
