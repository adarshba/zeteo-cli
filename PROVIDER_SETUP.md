# AI Provider Setup Guide

This guide provides detailed instructions for setting up each AI provider supported by Zeteo CLI.

## Quick Start with .env File

**The easiest way to configure all providers is using a `.env` file:**

```bash
# 1. Copy the example file in your zeteo-cli directory
cp .env.example .env

# 2. Edit the .env file with your credentials
nano .env

# 3. Add your API keys (example):
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxx
GOOGLE_API_KEY=AIzaSyxxxxxxxxxxxxxxx
GOOGLE_CLOUD_PROJECT=my-project-id
AZURE_OPENAI_API_KEY=xxxxxxxxxxxxxxxx
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_DEPLOYMENT=gpt-4-deployment

# 4. Run zeteo - it will automatically load your .env file!
zeteo chat "Hello!"
```

**Benefits of using .env:**
- ✅ No need to export variables manually
- ✅ All credentials in one place
- ✅ Easy to switch between providers
- ✅ Automatically loaded on every run
- ✅ Safe (automatically ignored by git)

**Note:** The `.env` file should be in the same directory where you run the `zeteo` command.

---

## Table of Contents

- [Quick Start with .env File](#quick-start-with-env-file)
- [OpenAI](#openai)
- [Google AI (Gemini)](#google-ai-gemini)
- [Vertex AI](#vertex-ai)
- [Azure OpenAI](#azure-openai)
- [Quick Comparison](#quick-comparison)
- [Troubleshooting](#troubleshooting)

---

## OpenAI

### Overview
OpenAI provides access to GPT models including GPT-4 and GPT-3.5-turbo through their API.

### Prerequisites
- OpenAI account at https://platform.openai.com
- Valid payment method (credit card)
- API key

### Cost
- Pay-as-you-go pricing
- GPT-4: ~$0.03 per 1K input tokens, ~$0.06 per 1K output tokens
- GPT-3.5-turbo: ~$0.0015 per 1K input tokens, ~$0.002 per 1K output tokens

### Setup Steps

1. **Create Account**
   - Visit https://platform.openai.com
   - Sign up or log in

2. **Add Payment Method**
   - Go to Settings → Billing
   - Add a credit card

3. **Generate API Key**
   ```bash
   # Visit https://platform.openai.com/api-keys
   # Click "Create new secret key"
   # Give it a name (e.g., "zeteo-cli")
   # Copy the key (starts with "sk-")
   ```

4. **Configure Environment**
   
   **Option A: Using .env file (Recommended)**
   ```bash
   # Add to .env file in your project directory
   echo 'OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxx' >> .env
   ```
   
   **Option B: Export manually**
   ```bash
   export OPENAI_API_KEY="sk-proj-xxxxxxxxxxxxx"
   ```

5. **Persist Configuration (if using manual export)**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   echo 'export OPENAI_API_KEY="sk-proj-xxxxxxxxxxxxx"' >> ~/.bashrc
   source ~/.bashrc
   ```

6. **Test**
   ```bash
   zeteo chat "Hello, OpenAI!"
   zeteo --provider openai chat "What is OpenTelemetry?"
   ```

### Default Model
- `gpt-4o` (optimized GPT-4)

### Supported Models
- gpt-4o
- gpt-4
- gpt-4-turbo
- gpt-3.5-turbo

---

## Google AI (Gemini)

### Overview
Google AI provides access to Gemini models through a simple API key, perfect for quick setup and testing.

### Prerequisites
- Google account
- Access to Google AI Studio

### Cost
- Free tier available with rate limits
- Pay-as-you-go for higher usage
- Gemini Pro: Free tier includes 60 requests per minute

### Setup Steps

1. **Access Google AI Studio**
   ```bash
   # Visit https://aistudio.google.com/app/apikey
   ```

2. **Create API Key**
   - Click "Get API key" or "Create API key"
   - Select or create a Google Cloud project
   - Copy the API key (starts with "AIza")

3. **Configure Environment**
   ```bash
   export GOOGLE_API_KEY="AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
   ```

4. **Persist Configuration**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   echo 'export GOOGLE_API_KEY="AIzaSyXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"' >> ~/.bashrc
   source ~/.bashrc
   ```

5. **Test**
   ```bash
   zeteo chat --provider google "Hello, Gemini!"
   zeteo --provider google "Explain observability"
   ```

### Default Model
- `gemini-pro`

### Supported Models
- gemini-pro
- gemini-1.5-pro

### Differences from Vertex AI
- **Google AI**: Simple API key, good for development and testing
- **Vertex AI**: Enterprise features, better for production, requires GCP setup

---

## Vertex AI

### Overview
Vertex AI is Google Cloud's enterprise AI platform, offering Gemini models with production-grade features like monitoring, quotas, and compliance.

### Prerequisites
- Google Cloud Platform account
- Active GCP project with billing enabled
- gcloud CLI installed
- Vertex AI API enabled

### Cost
- Pay-as-you-go pricing
- Gemini Pro: ~$0.00025 per 1K characters input, ~$0.0005 per 1K characters output
- Free tier: $300 credit for new GCP accounts

### Setup Steps

1. **Install gcloud CLI**
   
   **macOS:**
   ```bash
   brew install --cask google-cloud-sdk
   ```
   
   **Linux:**
   ```bash
   curl https://sdk.cloud.google.com | bash
   exec -l $SHELL
   gcloud init
   ```
   
   **Windows:**
   ```powershell
   # Download from https://cloud.google.com/sdk/docs/install
   # Run the installer
   ```

2. **Authenticate**
   ```bash
   # Login to Google Cloud
   gcloud auth login
   
   # Set up application default credentials (required for Zeteo)
   gcloud auth application-default login
   ```

3. **Create/Select Project**
   ```bash
   # List existing projects
   gcloud projects list
   
   # Create new project (optional)
   gcloud projects create my-zeteo-project --name="Zeteo AI"
   
   # Set active project
   gcloud config set project my-zeteo-project
   ```

4. **Enable Vertex AI API**
   ```bash
   gcloud services enable aiplatform.googleapis.com
   ```

5. **Configure Environment**
   ```bash
   export GOOGLE_CLOUD_PROJECT="my-zeteo-project"
   export GOOGLE_CLOUD_LOCATION="us-central1"  # optional, defaults to us-central1
   ```

6. **Persist Configuration**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   echo 'export GOOGLE_CLOUD_PROJECT="my-zeteo-project"' >> ~/.bashrc
   echo 'export GOOGLE_CLOUD_LOCATION="us-central1"' >> ~/.bashrc
   source ~/.bashrc
   ```

7. **Test**
   ```bash
   # Verify authentication
   gcloud auth application-default print-access-token
   
   # Test Zeteo
   zeteo chat --provider vertex "Hello, Vertex AI!"
   ```

### Default Model
- `gemini-pro`

### Available Regions
- us-central1 (Iowa)
- us-east1 (South Carolina)
- us-west1 (Oregon)
- europe-west1 (Belgium)
- asia-northeast1 (Tokyo)

### Troubleshooting Vertex AI

**Problem: "Failed to get access token"**
```bash
# Solution: Re-authenticate
gcloud auth application-default login
```

**Problem: "Permission denied"**
```bash
# Solution: Enable API and check permissions
gcloud services enable aiplatform.googleapis.com
gcloud projects get-iam-policy $GOOGLE_CLOUD_PROJECT
```

**Problem: "Project not set"**
```bash
# Solution: Set project explicitly
gcloud config set project my-zeteo-project
export GOOGLE_CLOUD_PROJECT="my-zeteo-project"
```

---

## Azure OpenAI

### Overview
Azure OpenAI Service provides access to OpenAI models through Microsoft Azure infrastructure, offering enterprise features like private networking and compliance.

### Prerequisites
- Azure account with active subscription
- Azure OpenAI resource created
- Model deployed

### Cost
- Pay-as-you-go pricing through Azure
- GPT-4: Similar pricing to OpenAI (~$0.03-0.06 per 1K tokens)
- Billed through Azure subscription

### Setup Steps

1. **Create Azure Account**
   ```bash
   # Visit https://azure.microsoft.com
   # Sign up for free account ($200 credit)
   ```

2. **Request Access to Azure OpenAI**
   ```bash
   # Visit https://aka.ms/oai/access
   # Fill out the form
   # Wait for approval (usually 1-2 business days)
   ```

3. **Create Azure OpenAI Resource**
   
   **Via Azure Portal:**
   - Go to https://portal.azure.com
   - Search for "Azure OpenAI"
   - Click "Create"
   - Fill in:
     - Subscription
     - Resource group (create new or use existing)
     - Region (e.g., East US, West Europe)
     - Name (e.g., "my-zeteo-openai")
     - Pricing tier (Standard S0)
   - Click "Review + create" → "Create"
   - Wait for deployment (~2-5 minutes)

4. **Deploy a Model**
   
   - Navigate to your Azure OpenAI resource
   - Click "Model deployments" or go to Azure OpenAI Studio
   - Click "Create new deployment"
   - Select:
     - Model: gpt-4, gpt-35-turbo, or gpt-4-turbo
     - Deployment name: e.g., "gpt-4-deployment"
   - Click "Create"

5. **Get Credentials**
   
   - In your Azure OpenAI resource
   - Go to "Keys and Endpoint"
   - Copy:
     - KEY 1 (or KEY 2)
     - Endpoint URL

6. **Configure Environment**
   ```bash
   export AZURE_OPENAI_API_KEY="1234567890abcdef1234567890abcdef"
   export AZURE_OPENAI_ENDPOINT="https://my-zeteo-openai.openai.azure.com"
   export AZURE_OPENAI_DEPLOYMENT="gpt-4-deployment"
   ```

7. **Persist Configuration**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   echo 'export AZURE_OPENAI_API_KEY="1234567890abcdef1234567890abcdef"' >> ~/.bashrc
   echo 'export AZURE_OPENAI_ENDPOINT="https://my-zeteo-openai.openai.azure.com"' >> ~/.bashrc
   echo 'export AZURE_OPENAI_DEPLOYMENT="gpt-4-deployment"' >> ~/.bashrc
   source ~/.bashrc
   ```

8. **Test**
   ```bash
   # Test endpoint connectivity
   curl -i $AZURE_OPENAI_ENDPOINT/openai/deployments?api-version=2024-02-15-preview \
     -H "api-key: $AZURE_OPENAI_API_KEY"
   
   # Test Zeteo
   zeteo chat --provider azure "Hello, Azure OpenAI!"
   ```

### API Version
- Zeteo uses API version: `2024-02-15-preview`
- Automatically configured, no manual setup needed

### Supported Models
- gpt-4
- gpt-4-turbo
- gpt-35-turbo
- gpt-35-turbo-16k

### Troubleshooting Azure OpenAI

**Problem: "Endpoint not found"**
```bash
# Verify endpoint format
echo $AZURE_OPENAI_ENDPOINT
# Should be: https://your-resource-name.openai.azure.com
# NOT: https://your-resource-name.openai.azure.com/
```

**Problem: "Deployment not found"**
```bash
# List deployments
curl -i $AZURE_OPENAI_ENDPOINT/openai/deployments?api-version=2024-02-15-preview \
  -H "api-key: $AZURE_OPENAI_API_KEY"

# Verify deployment name matches
echo $AZURE_OPENAI_DEPLOYMENT
```

**Problem: "Access denied"**
```bash
# Check if access was approved
# Visit https://aka.ms/oai/access and check status
# Ensure your Azure subscription has Azure OpenAI enabled
```

---

## Quick Comparison

| Feature | OpenAI | Google AI | Vertex AI | Azure OpenAI |
|---------|--------|-----------|-----------|--------------|
| **Setup Complexity** | Easy | Easy | Medium | Medium |
| **Cost** | Pay-per-use | Free tier + paid | Pay-per-use | Pay-per-use |
| **Free Tier** | No | Yes | $300 credit | $200 credit |
| **Auth Method** | API Key | API Key | gcloud + project | API Key + Endpoint |
| **Best For** | Production | Development | Enterprise GCP | Enterprise Azure |
| **Model Selection** | GPT-4, GPT-3.5 | Gemini Pro | Gemini Pro | GPT-4, GPT-3.5 |
| **Setup Time** | 5 minutes | 2 minutes | 15 minutes | 20 minutes |

### Which Provider Should I Choose?

**Choose OpenAI if:**
- You want the simplest setup
- You're already using OpenAI elsewhere
- You need GPT-4 or GPT-3.5 specifically

**Choose Google AI if:**
- You want to try Gemini quickly
- You're developing/testing
- You want a free tier

**Choose Vertex AI if:**
- You're using Google Cloud Platform
- You need enterprise features (monitoring, quotas)
- You want production-grade deployment

**Choose Azure OpenAI if:**
- You're using Microsoft Azure
- You need private network access
- You need Azure compliance features

---

## Troubleshooting

### General Issues

**"API key not set"**
```bash
# Check which provider
zeteo chat --provider <provider> "test"

# Verify environment variables
env | grep -E "(OPENAI|GOOGLE|AZURE)"

# Set appropriate variable
export OPENAI_API_KEY="..."      # OpenAI
export GOOGLE_API_KEY="..."      # Google AI
export GOOGLE_CLOUD_PROJECT="..." # Vertex AI
export AZURE_OPENAI_API_KEY="..." # Azure OpenAI
```

**"Unknown provider"**
```bash
# Valid provider names are:
openai
google
vertex
azure

# Usage:
zeteo chat --provider openai "test"
```

**"Connection timeout"**
```bash
# Check internet connection
ping -c 3 api.openai.com
ping -c 3 generativelanguage.googleapis.com

# Try with verbose flag
zeteo --verbose chat --provider openai "test"
```

### Provider-Specific Issues

See individual provider sections above for detailed troubleshooting.

### Getting Help

1. Check environment variables: `env | grep -E "(OPENAI|GOOGLE|AZURE)"`
2. Run with verbose flag: `zeteo --verbose chat "test"`
3. Check [GitHub Issues](https://github.com/adarshba/zeteo-cli/issues)
4. Review provider documentation:
   - [OpenAI Docs](https://platform.openai.com/docs)
   - [Google AI Docs](https://ai.google.dev/docs)
   - [Vertex AI Docs](https://cloud.google.com/vertex-ai/docs)
   - [Azure OpenAI Docs](https://learn.microsoft.com/azure/ai-services/openai/)

---

## Security Best Practices

### API Key Storage

**❌ Don't:**
- Commit API keys to git
- Share API keys in public channels
- Store keys in code files

**✅ Do:**
- Use environment variables
- Store in secure key management systems
- Rotate keys regularly
- Use `.env` files (add to `.gitignore`)

### Environment Variables

```bash
# Create .env file (DON'T commit this)
cat > .env << EOF
OPENAI_API_KEY="sk-..."
GOOGLE_API_KEY="AIza..."
AZURE_OPENAI_API_KEY="..."
AZURE_OPENAI_ENDPOINT="https://..."
AZURE_OPENAI_DEPLOYMENT="..."
EOF

# Load environment variables
source .env

# Or use direnv
echo "source .env" > .envrc
direnv allow
```

### Key Rotation

```bash
# OpenAI: Rotate in platform.openai.com/api-keys
# Google AI: Rotate in console.cloud.google.com
# Vertex AI: Rotate via gcloud auth
# Azure: Rotate in portal.azure.com
```

---

## .env File Best Practices

### Using .env Files with Zeteo

Zeteo automatically loads environment variables from a `.env` file in your current directory.

**Setup:**
```bash
# 1. Copy example file
cp .env.example .env

# 2. Edit with your credentials
nano .env

# 3. That's it! Zeteo will load it automatically
zeteo chat "test"
```

**Location:**
- Place `.env` in the directory where you run `zeteo`
- Or use a global location and source it in your shell profile

**Example .env file:**
```bash
# .env
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxx
GOOGLE_API_KEY=AIzaSyxxxxxxxxxxxxxxx
GOOGLE_CLOUD_PROJECT=my-gcp-project
GOOGLE_CLOUD_LOCATION=us-central1
AZURE_OPENAI_API_KEY=xxxxxxxxxxxxxxxx
AZURE_OPENAI_ENDPOINT=https://my-resource.openai.azure.com
AZURE_OPENAI_DEPLOYMENT=gpt-4-deployment
```

### Verbose Mode

Use `--verbose` to see when the .env file is loaded:

```bash
$ zeteo --verbose chat "test"
Verbose mode enabled
Loaded environment variables from .env file
Using AI provider: openai
...
```

### Multiple .env Files

You can use different .env files for different projects:

```bash
# Project A directory
cd ~/projects/project-a
cat > .env << EOF
OPENAI_API_KEY=sk-project-a-key
EOF
zeteo chat "test"  # Uses project-a key

# Project B directory
cd ~/projects/project-b
cat > .env << EOF
OPENAI_API_KEY=sk-project-b-key
EOF
zeteo chat "test"  # Uses project-b key
```

### Backup and Version Control

**Important:**
- ✅ Commit `.env.example` (template without real keys)
- ❌ **Never** commit `.env` (contains real keys)
- ✅ The `.gitignore` file already excludes `.env`

**Backup your .env safely:**
```bash
# Encrypt before backing up
gpg -c .env  # Creates .env.gpg
# Store .env.gpg in secure backup location

# Restore
gpg .env.gpg  # Decrypts to .env
```

### Troubleshooting .env Files

**Problem: .env not being loaded**
```bash
# Check if file exists
ls -la .env

# Check file content (remove after checking!)
cat .env

# Run with verbose to confirm loading
zeteo --verbose chat "test"
```

**Problem: Variables not taking effect**
```bash
# Make sure there are no spaces around =
# ❌ Wrong: OPENAI_API_KEY = sk-xxx
# ✅ Correct: OPENAI_API_KEY=sk-xxx

# Check for quotes
# Both work:
OPENAI_API_KEY=sk-xxx
OPENAI_API_KEY="sk-xxx"
```

**Problem: Permission issues**
```bash
# Set appropriate permissions (readable only by you)
chmod 600 .env
```

---

*For more information, see the main [README.md](README.md) and [examples/REPL_GUIDE.md](examples/REPL_GUIDE.md)*
