# API Key Setup Guide

## Quick Setup

### 1. Create `.env` file

```bash
cp .env.example .env
```

### 2. Add your API key

Edit `.env` and replace `your_key_here` with your actual key.

### 3. Load environment variables

**Option A: Export manually**
```bash
export ETHERSCAN_API_KEY=your_key_here
```

**Option B: Use direnv (recommended)**
```bash
# Install direnv
sudo apt install direnv  # Ubuntu/Debian
brew install direnv      # macOS

# Add to shell
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc  # bash
echo 'eval "$(direnv hook zsh)"' >> ~/.zshrc    # zsh

# Allow directory
direnv allow .
```

**Option C: Source .env**
```bash
export $(grep -v '^#' .env | xargs)
```

### 4. Test the setup

```bash
scpf scan 0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984 --chain ethereum
```

## Security Notes

✅ **DO:**
- Keep `.env` file local (already in `.gitignore`)
- Use `.env.example` as template for team
- Rotate API keys periodically

❌ **DON'T:**
- Commit `.env` to git
- Share API keys in chat/email
- Use production keys for testing

## Getting API Keys

| Service | URL |
|---------|-----|
| Etherscan | https://etherscan.io/apis |
| BscScan | https://bscscan.com/apis |
| PolygonScan | https://polygonscan.com/apis |
| Arbiscan | https://arbiscan.io/apis |
| Optimistic Etherscan | https://optimistic.etherscan.io/apis |
| Basescan | https://basescan.org/apis |

## Current Setup

Your Etherscan API key should be configured in `.env` file (not tracked by git).

This is a **free tier** key with rate limits:
- 5 calls/second
- 100,000 calls/day

## Troubleshooting

### "API key not found"
```bash
# Check if variable is set
echo $ETHERSCAN_API_KEY

# If empty, export it
export ETHERSCAN_API_KEY=your_key_here
```

### "Rate limit exceeded"
- Wait a few seconds between scans
- Consider upgrading to paid tier
- Use `--concurrency 1` flag

### "Invalid API key"
- Verify key at https://etherscan.io/myapikey
- Check for typos in `.env`
- Ensure no extra spaces
