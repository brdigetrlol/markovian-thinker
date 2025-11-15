# 🏃 GitHub Actions Workflows

This repository uses GitHub Actions for automated code production, testing, and deployment.

---

## 📋 Available Workflows

### 🧪 **Testing & Quality**

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `auto-test.yml` | Push, PR | Run all tests, linting, formatting checks |
| `auto-format.yml` | Push to `claude/**` | Auto-format code with rustfmt/prettier |
| `auto-security.yml` | Daily, Push | Security audits, vulnerability scanning |
| `ai-code-assistant.yml` | PR | AI-powered code review and suggestions |

### 🚀 **Deployment**

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `auto-deploy.yml` | Push to `main` | Build, test, and deploy to production |
| `auto-docs.yml` | Push to `main` | Generate and publish documentation |

### 🔄 **Maintenance**

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `auto-update-deps.yml` | Weekly, Manual | Update Rust/npm dependencies |

---

## 🎯 Quick Start

### 1. **Enable GitHub Actions**

Go to your repo → Settings → Actions → Allow all actions

### 2. **Add Secrets** (if needed)

Settings → Secrets and variables → Actions → New repository secret

Required secrets for AI features:
- `OPENAI_API_KEY` - For AI code review (optional)

### 3. **Trigger Workflows**

#### Automatic:
- **Push code** → Tests run automatically
- **Open PR** → Code review + tests run
- **Merge to main** → Deploy automatically

#### Manual:
Comment on PR or issue:
- `/generate-tests` - Generate test cases
- `/refactor` - Suggest refactoring

Or run manually:
- Go to Actions tab
- Select workflow
- Click "Run workflow"

---

## 🛠️ Self-Hosted Runner Setup

### Why Self-Host?

- ✅ **Unlimited minutes** (GitHub-hosted: 2,000/month free)
- ✅ **Faster builds** (your hardware, your cache)
- ✅ **Custom environment** (pre-install tools)
- ✅ **Private network access** (deploy to internal servers)

### Quick Setup:

```bash
# Run the setup script
./.github/setup-runner.sh

# Follow the prompts
# Your runner will appear in Settings → Actions → Runners
```

### Manual Setup:

1. Go to: Settings → Actions → Runners → New self-hosted runner
2. Follow the commands shown (personalized for your repo)
3. Install as service to run on boot:
   ```bash
   sudo ./svc.sh install
   sudo ./svc.sh start
   ```

---

## 🎨 Customization

### Change When Workflows Run:

Edit the `on:` section in any `.yml` file:

```yaml
on:
  push:
    branches: [ main ]     # Only on main
  pull_request:            # On all PRs
  schedule:
    - cron: '0 9 * * 1'   # Every Monday 9 AM
  workflow_dispatch:       # Manual trigger
```

### Use Self-Hosted Runner:

Change `runs-on:` in any workflow:

```yaml
jobs:
  my-job:
    runs-on: self-hosted  # Instead of ubuntu-latest
```

### Add Environment Variables:

```yaml
env:
  RUST_LOG: debug
  NODE_ENV: production
```

---

## 📊 Workflow Status

Check status at: `https://github.com/YOUR_USERNAME/YOUR_REPO/actions`

### Status Badges:

Add to README.md:

```markdown
![Tests](https://github.com/YOUR_USERNAME/YOUR_REPO/workflows/Auto%20Test%20%26%20Lint/badge.svg)
![Deploy](https://github.com/YOUR_USERNAME/YOUR_REPO/workflows/Auto%20Deploy/badge.svg)
```

---

## 🔒 Security Best Practices

1. **Never commit secrets** - Use GitHub Secrets
2. **Review dependencies** - Auto-scan enabled
3. **Pin action versions** - Use `@v3` not `@main`
4. **Limit permissions** - Set minimum required permissions
5. **Verify PRs** - Review before merging

---

## 💡 Advanced Examples

### Conditional Jobs:

```yaml
jobs:
  deploy:
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
```

### Matrix Testing:

```yaml
strategy:
  matrix:
    rust: [stable, beta, nightly]
    os: [ubuntu-latest, macos-latest, windows-latest]
```

### Dependent Jobs:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest

  test:
    needs: build
    runs-on: ubuntu-latest

  deploy:
    needs: [build, test]
    runs-on: ubuntu-latest
```

---

## 📚 Resources

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Actions Marketplace](https://github.com/marketplace?type=actions)
- [Self-hosted Runners](https://docs.github.com/en/actions/hosting-your-own-runners)
- [Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)

---

## 🆘 Troubleshooting

### Workflow not running?

1. Check if Actions are enabled: Settings → Actions
2. Check branch filters in workflow `.yml`
3. Check workflow syntax: `yamllint .github/workflows/*.yml`

### Runner not showing up?

1. Check runner is running: `./run.sh` or `sudo ./svc.sh status`
2. Check network connectivity
3. Regenerate token if expired

### Tests failing?

1. Check logs in Actions tab
2. Run locally to reproduce
3. Check dependencies are cached properly

---

## 🎯 Next Steps

1. **Enable workflows** - Push this repo to trigger them
2. **Add secrets** - For deployment/AI features
3. **Setup runner** - Run `./.github/setup-runner.sh`
4. **Monitor** - Watch Actions tab for results
5. **Customize** - Edit workflows for your needs

---

**All workflows are production-ready and tested!** 🚀
