# 🚀 Sentiment Intelligence Platform - Setup Guide

## Quick Start (3 Methods)

### Method 1: Docker (Recommended - Production Ready)
```bash
docker-compose up -d
open http://localhost:8080
```

### Method 2: Local Development
```bash
# Build TypeScript
./build.sh

# Start local server
npm run serve

# Open browser
open http://localhost:8080
```

### Method 3: Full Stack with Rust API
```bash
# Terminal 1: Start Rust API
cd ../rustml-sentiment-api/demo
./run-demo.sh

# Terminal 2: Build and serve frontend
./build.sh
npm run serve
```

---

## Prerequisites

### For Docker Deployment:
- Docker 20.10+
- Docker Compose 1.29+

### For Local Development:
- Node.js 16+ and npm 8+
- TypeScript 5.0+
- Python 3.7+ (for local server)

### For Full Stack:
- All of the above
- Rust 1.70+
- Cargo

---

## Installation Steps

### 1. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# This installs:
# - TypeScript compiler
# - Three.js (3D visualization)
# - Chart.js (2D charts)
# - Type definitions
```

### 2. Build TypeScript

```bash
# One-time build
npm run build

# Watch mode (auto-rebuild on changes)
npm run build:watch

# Type check only (no output)
npm run type-check
```

### 3. Run the Application

```bash
# Option A: Simple HTTP server
npm run serve

# Option B: Custom server (if you have one)
python3 -m http.server 8080

# Option C: Use any static server
npx http-server -p 8080
```

### 4. Access the Dashboard

Open your browser and navigate to:
```
http://localhost:8080
```

---

## Directory Structure

```
sentiment-intelligence-platform/
├── index.html              # Main dashboard (production UI)
├── app.ts                  # TypeScript source code
├── dist/                   # Compiled JavaScript (generated)
│   ├── app.js
│   ├── app.d.ts
│   └── app.js.map
├── package.json            # Node.js dependencies
├── tsconfig.json           # TypeScript configuration
├── build.sh                # Build script (automated)
├── docker-compose.yml      # Docker deployment
├── README.md               # Business documentation
└── SETUP.md                # This file
```

---

## TypeScript Configuration

The `tsconfig.json` is configured for:

- **Target**: ES2020 (modern JavaScript)
- **Module**: ES2020 (native modules)
- **Strict Mode**: Enabled (type safety)
- **Source Maps**: Enabled (debugging)
- **Output**: `dist/` directory

### Custom TypeScript Build

```bash
# Standard build
tsc

# Watch mode
tsc --watch

# Specific file
tsc app.ts

# Check types only (no output)
tsc --noEmit
```

---

## Connecting to Rust API Backend

The platform can connect to the real Rust sentiment analysis API:

### 1. Start the Rust API

```bash
cd ../rustml-sentiment-api/demo
./run-demo.sh
```

This starts:
- Rust API on `http://localhost:3000`
- Frontend on `http://localhost:8080`

### 2. Enable API Mode

In `app.ts`, the platform automatically detects the API:

```typescript
// Automatic detection
async checkAPIStatus() {
    const response = await fetch('http://localhost:3000/api/v1/health');
    if (response.ok) {
        CONFIG.USE_REAL_API = true; // Switches to real API
    }
}
```

When the API is available:
- ✅ Real sentiment analysis (VADER ML algorithm)
- ✅ Text classification into 11+ categories
- ✅ Sub-5ms response times
- ✅ Production-ready Rust performance

When the API is NOT available:
- ℹ️ Demo mode (simulated data)
- ℹ️ Full UI functionality
- ℹ️ Perfect for portfolio demonstrations

---

## Development Workflow

### 1. Edit TypeScript

```bash
# Open in your editor
code app.ts

# Or vim, nano, etc.
vim app.ts
```

### 2. Auto-rebuild

```bash
# In one terminal
npm run build:watch
```

### 3. Serve

```bash
# In another terminal
npm run serve
```

### 4. Test

Open `http://localhost:8080` and see changes live!

---

## Production Deployment

### Option 1: Static Hosting

Deploy to any static host:

**Netlify:**
```bash
# Build
npm run build

# Deploy
netlify deploy --prod --dir=.
```

**Vercel:**
```bash
npm run build
vercel --prod
```

**GitHub Pages:**
```bash
npm run build
git add dist/
git commit -m "Build production assets"
git push origin main
```

### Option 2: Docker Container

```bash
# Build image
docker build -t sentiment-platform .

# Run container
docker run -p 8080:80 sentiment-platform

# Or use Docker Compose
docker-compose up -d
```

### Option 3: Cloud Platforms

**AWS S3 + CloudFront:**
```bash
npm run build
aws s3 sync . s3://your-bucket --exclude "*.ts" --exclude "node_modules/*"
aws cloudfront create-invalidation --distribution-id YOUR_ID --paths "/*"
```

**Google Cloud Storage:**
```bash
npm run build
gsutil -m rsync -r -x "node_modules|*.ts" . gs://your-bucket
```

**Azure Static Web Apps:**
```bash
npm run build
# Configure via Azure Portal or CLI
```

---

## Environment Variables

Create a `.env` file (optional):

```bash
# API Configuration
API_URL=http://localhost:3000/api/v1
USE_REAL_API=false

# Update Intervals (milliseconds)
UPDATE_INTERVAL=2000

# Visualization Limits
MAX_FEED_ITEMS=50
MAX_3D_POINTS=200
```

---

## Troubleshooting

### TypeScript Compilation Errors

**Problem**: `Cannot find module 'three'`
**Solution**:
```bash
npm install
npm install --save-dev @types/three
```

**Problem**: `Property 'xyz' does not exist on type`
**Solution**: Check `tsconfig.json` has `"strict": true` and fix type issues

### API Connection Issues

**Problem**: "API unavailable, using simulation"
**Solution**:
1. Check Rust API is running: `curl http://localhost:3000/health`
2. Check CORS is enabled in API
3. Verify API URL in `app.ts`

### 3D Visualization Not Rendering

**Problem**: Black screen instead of 3D view
**Solution**:
1. Check browser console for errors
2. Verify Three.js loaded: Check Network tab
3. Try different browser (Chrome recommended)
4. Check WebGL support: Visit `https://get.webgl.org/`

### Charts Not Displaying

**Problem**: Empty chart containers
**Solution**:
1. Verify Chart.js loaded
2. Check canvas elements exist: `<canvas id="trendChart">`
3. Ensure `dist/app.js` is compiled and loaded

---

## Performance Optimization

### For Production:

1. **Minify JavaScript:**
   ```bash
   npm install --save-dev terser
   terser dist/app.js -o dist/app.min.js --compress --mangle
   ```

2. **Enable Gzip:**
   ```nginx
   gzip on;
   gzip_types text/javascript application/javascript;
   ```

3. **CDN for Libraries:**
   Already configured in `index.html`:
   - Three.js from CDN
   - Chart.js from CDN

4. **Lazy Loading:**
   Consider code splitting for large apps:
   ```typescript
   const module = await import('./heavy-module.js');
   ```

---

## Testing

### Type Safety Check:
```bash
npm run type-check
```

### Browser Compatibility:
- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

### Performance:
- Target: 60 FPS for 3D visualization
- Target: < 100ms for data updates
- Target: < 5ms for API responses (with Rust backend)

---

## Next Steps

1. **Customize**: Edit `app.ts` to add your own features
2. **Deploy**: Choose a deployment method above
3. **Integrate**: Connect to real data sources (Twitter, reviews, etc.)
4. **Monetize**: Add authentication and billing (Stripe)

---

## Support

**Issues**: https://github.com/brdigetrlol/icarus-core/issues
**Email**: cody.moore@outlook.com
**Portfolio**: This project demonstrates production-ready TypeScript/Rust development

---

## License

MIT License - Free to use, modify, and deploy commercially.
