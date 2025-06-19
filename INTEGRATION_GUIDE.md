# Integration Guide: Using ScreenCaptureKit Rust in WhisperDesk

This guide shows how to integrate the standalone ScreenCaptureKit Rust package into your WhisperDesk project.

## 🚀 Quick Integration Steps

### 1. Create the Separate Repository

```bash
# Create a new repository on GitHub (or your preferred platform)
# Example: https://github.com/your-username/screencapturekit-rust

# Clone and set up the repository
git clone https://github.com/your-username/screencapturekit-rust.git
cd screencapturekit-rust

# Copy all files from /tmp/screencapturekit-rust/ to this directory
# (You've already consolidated all files in /tmp/screencapturekit-rust/)

# Initialize the repository
git add .
git commit -m "Initial commit: ScreenCaptureKit Rust with bypass approach"
git push origin main
```

### 2. Publish to NPM (Recommended)

```bash
# In your screencapturekit-rust repository
cd screencapturekit-rust

# Install dependencies
npm install

# Build the native module
npm run build

# Login to NPM (if not already logged in)
npm login

# Publish the package
npm publish --access public
```

### 3. Install in WhisperDesk Project

```bash
# In your WhisperDesk project directory
cd /path/to/WhisperDesk

# Install the published package
npm install @whisperdesk/screencapturekit-rust

# Or install directly from GitHub (if not published to NPM)
npm install git+https://github.com/your-username/screencapturekit-rust.git
```

## 🔧 Integration Options

### Option A: Published NPM Package (Recommended)

**Pros:**
- ✅ Easy to install and update
- ✅ Version management through NPM
- ✅ Can be used in multiple projects
- ✅ Automatic dependency resolution

**Steps:**
1. Publish to NPM as `@whisperdesk/screencapturekit-rust`
2. Install in WhisperDesk: `npm install @whisperdesk/screencapturekit-rust`
3. Import and use: `const { ScreenCaptureKitRecorder } = require('@whisperdesk/screencapturekit-rust')`

### Option B: Git Dependency

**Pros:**
- ✅ No need for NPM publishing
- ✅ Direct from source control
- ✅ Can use specific branches/tags

**Steps:**
1. Push to GitHub repository
2. Install in WhisperDesk: `npm install git+https://github.com/your-username/screencapturekit-rust.git`
3. Import and use normally

### Option C: Local Development Link

**Pros:**
- ✅ Great for development
- ✅ Real-time changes
- ✅ No publishing needed

**Steps:**
1. In screencapturekit-rust directory: `npm link`
2. In WhisperDesk directory: `npm link @whisperdesk/screencapturekit-rust`

## 📝 Update WhisperDesk Code

### Replace Current Implementation

In your WhisperDesk project, replace the current native module usage:

**Before:**
```javascript
// Old internal module
const { ScreenCaptureKitRecorder } = require('./native/whisperdesk-screencapturekit');
```

**After:**
```javascript
// New standalone package
const { ScreenCaptureKitRecorder } = require('@whisperdesk/screencapturekit-rust');
```

### Update Package.json

Add the dependency to your WhisperDesk `package.json`:

```json
{
  "dependencies": {
    "@whisperdesk/screencapturekit-rust": "^1.0.0",
    // ... other dependencies
  }
}
```

### Remove Native Directory

Once integrated, you can remove the old native implementation:

```bash
# In WhisperDesk project
rm -rf native/whisperdesk-screencapturekit
```

## 🧪 Testing Integration

Create a test file to verify the integration works:

```javascript
// test-integration.js
const { ScreenCaptureKitRecorder, checkScreenRecordingPermission } = require('@whisperdesk/screencapturekit-rust');

console.log('Testing ScreenCaptureKit Rust integration...');

// Test permission
const hasPermission = checkScreenRecordingPermission();
console.log('Screen recording permission:', hasPermission);

// Test screen enumeration
const recorder = new ScreenCaptureKitRecorder();
const screens = recorder.getAvailableScreensWithTimeout(5000);
console.log(`Found ${screens.length} screens`);

console.log('✅ Integration test successful!');
```

Run the test:
```bash
node test-integration.js
```

## 🔄 Development Workflow

### For ScreenCaptureKit Package Development

```bash
# In screencapturekit-rust repository
git clone https://github.com/your-username/screencapturekit-rust.git
cd screencapturekit-rust

# Make changes to the Rust code
# Build and test
npm run build
npm test

# Commit and push changes
git add .
git commit -m "Improve bypass approach"
git push origin main

# Publish new version (if using NPM)
npm version patch  # or minor/major
npm publish
```

### For WhisperDesk Integration

```bash
# In WhisperDesk project
# Update to latest version
npm update @whisperdesk/screencapturekit-rust

# Or install specific version
npm install @whisperdesk/screencapturekit-rust@1.0.1
```

## 📦 Build Configuration

### For CI/CD (GitHub Actions Example)

Create `.github/workflows/build.yml` in screencapturekit-rust repository:

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: macos-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Install dependencies
      run: npm install
      
    - name: Build native module
      run: npm run build
      
    - name: Run tests
      run: npm test
```

## 🚀 Benefits of This Approach

### ✅ Separation of Concerns
- **ScreenCaptureKit package**: Focused on screen capture functionality
- **WhisperDesk**: Focused on UI and application logic

### ✅ Reusability
- Can be used in other projects
- Community can contribute improvements
- Easier to maintain and test

### ✅ Version Management
- Independent versioning
- Semantic versioning support
- Easy rollbacks if needed

### ✅ Development Experience
- Faster builds (only rebuild when needed)
- Better IDE support
- Cleaner project structure

## 🔧 Troubleshooting

### Common Issues

1. **Build Failures**
   ```bash
   # Clean and rebuild
   rm -rf target/ node_modules/
   npm install
   npm run build
   ```

2. **Permission Issues**
   ```bash
   # Check permissions
   node -e "console.log(require('@whisperdesk/screencapturekit-rust').checkScreenRecordingPermission())"
   ```

3. **Import Errors**
   ```bash
   # Verify installation
   npm list @whisperdesk/screencapturekit-rust
   ```

### Support

- **Package Issues**: Create issues in the screencapturekit-rust repository
- **Integration Issues**: Check the integration guide and test files
- **macOS Compatibility**: Ensure you're running on macOS 10.15+

This integration approach provides a clean, maintainable solution while preserving all the bypass approach benefits you've developed! 