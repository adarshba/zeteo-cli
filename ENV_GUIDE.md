# Quick Start: Using .env for API Keys

This guide shows you how to use the `.env` file feature to manage your AI provider credentials easily.

## Why Use .env Files?

✅ **Convenient**: Set up once, use everywhere  
✅ **Secure**: Automatically excluded from git  
✅ **Simple**: No need to export variables manually  
✅ **Organized**: All credentials in one place  
✅ **Portable**: Easy to backup and restore  

## Setup (30 seconds)

### Step 1: Copy the Example File

```bash
cd /path/to/zeteo-cli
cp .env.example .env
```

### Step 2: Edit with Your Credentials

Open `.env` in your favorite editor:

```bash
nano .env
# or
vim .env
# or
code .env
```

Add your actual API keys:

```bash
# .env file
OPENAI_API_KEY=sk-proj-YOUR_REAL_KEY_HERE
GOOGLE_API_KEY=AIzaSyYOUR_REAL_KEY_HERE
# ... etc
```

### Step 3: Use Zeteo!

That's it! Zeteo will automatically load your `.env` file:

```bash
# No need to export anything!
zeteo chat "Hello, AI!"

# Try different providers
zeteo --provider google chat "What is observability?"
zeteo --provider azure chat "Explain OTEL"

# Start REPL mode
zeteo
zeteo --provider vertex
```

## Example .env File

Here's a complete example with all providers configured:

```bash
# .env
OPENAI_API_KEY=sk-proj-abc123def456ghi789jkl012mno345pqr678stu901
GOOGLE_API_KEY=AIzaSyABC123DEF456GHI789JKL012MNO345PQR678
GOOGLE_CLOUD_PROJECT=my-gcp-project-123456
GOOGLE_CLOUD_LOCATION=us-central1
AZURE_OPENAI_API_KEY=1234567890abcdef1234567890abcdef
AZURE_OPENAI_ENDPOINT=https://my-zeteo-ai.openai.azure.com
AZURE_OPENAI_DEPLOYMENT=gpt-4-deployment
```

Save this file, and you can instantly use any provider!

## Verification

Check if your .env file is being loaded:

```bash
# Run with verbose mode
zeteo --verbose chat "test"

# You should see:
# Verbose mode enabled
# Loaded environment variables from .env file
# Using AI provider: openai
```

## Multiple Projects

You can use different .env files for different projects:

```bash
# Project A
cd ~/projects/project-a
echo "OPENAI_API_KEY=sk-proj-key-for-project-a" > .env
zeteo chat "test"  # Uses project A's key

# Project B  
cd ~/projects/project-b
echo "OPENAI_API_KEY=sk-proj-key-for-project-b" > .env
zeteo chat "test"  # Uses project B's key
```

## Security

**Important:** The `.env` file is automatically excluded from git!

```bash
# ✅ This is already in .gitignore
.env
.env.local
.env.*.local

# ✅ DO commit this (template without real keys)
.env.example

# ❌ NEVER commit this (contains real keys)
.env
```

### Check Your .gitignore

```bash
cat .gitignore | grep .env
# Should show:
# .env
# .env.local
# .env.*.local
```

## Troubleshooting

### Problem: "API key not set"

**Solution 1:** Check if .env file exists
```bash
ls -la .env
```

**Solution 2:** Verify file content
```bash
cat .env
# Check for typos in variable names
```

**Solution 3:** Check for spaces
```bash
# ❌ Wrong (spaces around =)
OPENAI_API_KEY = sk-xxx

# ✅ Correct (no spaces)
OPENAI_API_KEY=sk-xxx
```

### Problem: .env not being loaded

**Solution 1:** Make sure you're in the right directory
```bash
pwd
ls -la .env
```

**Solution 2:** Check file permissions
```bash
chmod 600 .env
```

**Solution 3:** Verify with verbose mode
```bash
zeteo --verbose chat "test"
```

### Problem: Wrong credentials being used

**Solution:** Check for multiple environment variable sources
```bash
# Check if variables are already exported
env | grep -E "(OPENAI|GOOGLE|AZURE)"

# Unset if needed
unset OPENAI_API_KEY
unset GOOGLE_API_KEY
# etc.

# Now .env file will be used
zeteo chat "test"
```

## Advanced: Using direnv

For even more convenience, use [direnv](https://direnv.net/):

```bash
# Install direnv
# macOS:
brew install direnv

# Add to your shell (~/.bashrc or ~/.zshrc)
eval "$(direnv hook bash)"  # for bash
eval "$(direnv hook zsh)"   # for zsh

# Allow direnv for this directory
cd /path/to/zeteo-cli
direnv allow

# Now .env is automatically loaded when you cd into the directory!
```

## Backup Your .env Safely

**Never commit .env to git!** Instead, back it up securely:

```bash
# Option 1: Encrypt with GPG
gpg -c .env
# Creates .env.gpg (safe to backup)

# To restore:
gpg .env.gpg
# Decrypts back to .env

# Option 2: Store in password manager
# Copy content of .env to your password manager (1Password, LastPass, etc.)
```

## Summary

**Before (Manual Export):**
```bash
export OPENAI_API_KEY="sk-..."
export GOOGLE_API_KEY="AIza..."
export AZURE_OPENAI_API_KEY="..."
export AZURE_OPENAI_ENDPOINT="..."
export AZURE_OPENAI_DEPLOYMENT="..."
zeteo chat "test"
```

**After (With .env):**
```bash
# One-time setup
cp .env.example .env
nano .env  # Add your keys

# Every use (so simple!)
zeteo chat "test"
```

## Next Steps

1. ✅ Create your `.env` file from `.env.example`
2. ✅ Add your API keys
3. ✅ Test with: `zeteo --verbose chat "test"`
4. ✅ Start using: `zeteo` or `zeteo --provider <name>`

For detailed provider setup instructions, see [PROVIDER_SETUP.md](PROVIDER_SETUP.md).

---

**Need help?** Check the [main README](README.md) or [open an issue](https://github.com/adarshba/zeteo-cli/issues).
