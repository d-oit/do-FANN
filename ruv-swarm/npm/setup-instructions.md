# ruv-swarm Setup Instructions

## 🚀 Quick Setup (Recommended)

### Option 1: Automated Script (Easiest)

1. **Download the setup script:**
   ```bash
   # Clone the repository
   git clone https://github.com/ruvnet/ruv-FANN.git
   cd ruv-FANN/ruv-swarm/npm

   # Make script executable
   chmod +x build-setup-complete.sh
   ```

2. **Run the complete setup:**
   ```bash
   ./build-setup-complete.sh
   ```

   This script will:
   - ✅ Install Node.js 18.20.8+
   - ✅ Install system build tools (GCC, Clang)
   - ✅ Install Rust and wasm-pack
   - ✅ Install project dependencies
   - ✅ Build all WASM modules
   - ✅ Run tests and health checks
   - ✅ Create deployment package

### Option 2: Manual Step-by-Step Setup

If you prefer to do it manually or encounter issues with the script:

#### Step 1: Install Node.js
```bash
# Add NodeSource repository
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -

# Install Node.js
sudo apt-get update
sudo apt-get install -y nodejs

# Verify installation
node --version  # Should show v18.20.8 or higher
npm --version   # Should show 9.x or higher
```

#### Step 2: Install System Dependencies
```bash
# Install build essentials
sudo apt-get install -y build-essential gcc g++ clang

# Install additional dependencies
sudo apt-get install -y pkg-config libssl-dev curl git cmake python3-dev libsqlite3-dev
```

#### Step 3: Install Rust
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Reload shell environment
source ~/.cargo/env
export PATH="$HOME/.cargo/bin:$PATH"

# Verify installation
rustc --version  # Should show 1.85 or higher
cargo --version  # Should show cargo version
```

#### Step 4: Install wasm-pack
```bash
# Install wasm-pack
cargo install wasm-pack

# Verify installation
wasm-pack --version  # Should show 0.13.x
```

#### Step 5: Setup Project
```bash
# Clone repository (if not already done)
git clone https://github.com/ruvnet/ruv-FANN.git
cd ruv-FANN/ruv-swarm/npm

# Install dependencies
npm ci

# Run security audit
npm audit --audit-level moderate
```

#### Step 6: Build Project
```bash
# Build WASM modules
npm run build:wasm

# Build SIMD optimized WASM
npm run build:wasm-simd

# Build size-optimized WASM
npm run build:wasm-opt

# Build JavaScript bundle
npm run build
```

#### Step 7: Test and Verify
```bash
# Run all tests
npm run test:all

# Run health checks
node scripts/health-check.js

# Run performance tests
npm run test:performance
```

#### Step 8: Deploy
```bash
# Create deployment package
node scripts/deploy.js

# Or deploy with Docker
docker-compose up -d

# Or run directly
npm start
```

## 🐳 Docker-Based Setup (Alternative)

If you prefer using Docker instead of installing everything locally:

```bash
# Build Docker image
docker build -t ruv-swarm .

# Run container
docker run -d -p 3000:3000 --name ruv-swarm ruv-swarm

# Check logs
docker logs ruv-swarm

# Access application
open http://localhost:3000
```

## 🛠️ Troubleshooting

### Common Issues

#### 1. Permission Denied on Script
```bash
chmod +x build-setup-complete.sh
```

#### 2. Rust Installation Issues
```bash
# Remove existing Rust installation
rm -rf ~/.cargo ~/.rustup

# Reinstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 3. Build Failures
```bash
# Clear build cache
rm -rf ~/.cargo/registry
rm -rf target/

# Update Rust
rustup update

# Retry build
npm run build:all
```

#### 4. Test Failures
```bash
# Run specific test suite
npm run test:unit

# Check test logs
npm run test:all 2>&1 | tee test-output.log
```

## 📊 Verification Checklist

After setup, verify everything works:

- [ ] Node.js 18.20.8+ installed
- [ ] Rust 1.85+ installed
- [ ] wasm-pack installed
- [ ] All npm dependencies installed
- [ ] WASM modules built (check `wasm/` directory)
- [ ] JavaScript bundle built (check `dist/` directory)
- [ ] All tests pass
- [ ] Health checks pass
- [ ] Application starts successfully
- [ ] Can access http://localhost:3000

## 🎯 Next Steps After Setup

1. **Deploy to production** using the prepared scripts
2. **Set up monitoring** with the included monitoring stack
3. **Configure CI/CD** using the GitHub Actions workflows
4. **Customize configuration** for your specific environment

## 📞 Support

If you encounter issues:

1. Check the troubleshooting section above
2. Run health checks: `node scripts/health-check.js`
3. Check the logs and error messages
4. Review the deployment documentation: `DEPLOYMENT.md`
5. Create an issue on GitHub with setup logs

---

**Ready to get started? Run the setup script and you'll have a fully functional ruv-swarm deployment in minutes!** 🚀