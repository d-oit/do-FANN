# Production Deployment Guide

This guide covers deploying ruv-swarm to production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Manual Deployment](#manual-deployment)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Monitoring & Observability](#monitoring--observability)
- [Troubleshooting](#troubleshooting)
- [Rollback Procedures](#rollback-procedures)

## Prerequisites

### System Requirements

- **Node.js**: 18.20.8 or higher
- **Rust**: 1.85 or higher (for building WASM modules)
- **C Compiler**: GCC or Clang (for building dependencies)
- **Docker**: 20.10+ (for containerized deployment)
- **Memory**: Minimum 2GB RAM, 4GB recommended
- **Storage**: 1GB for application, plus space for logs and data

### Build Tools

```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y build-essential clang curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install wasm-pack
cargo install wasm-pack

# Install Node.js (if not already installed)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

## Quick Start

### Automated Deployment

```bash
# Navigate to the npm directory
cd ruv-swarm/npm

# Run the automated deployment script
node scripts/deploy.js
```

This script will:
- ✅ Check prerequisites
- ✅ Run quality checks
- ✅ Build all WASM modules
- ✅ Create deployment package
- ✅ Build Docker image
- ✅ Run health checks

### Manual Deployment

```bash
# 1. Install dependencies
npm ci

# 2. Run quality checks
npm run lint:check
npm run test:all

# 3. Build production artifacts
npm run build:all

# 4. Start the application
npm start
```

## Manual Deployment

### Step-by-Step Deployment

1. **Prepare the Environment**
   ```bash
   # Clone the repository
   git clone https://github.com/ruvnet/ruv-FANN.git
   cd ruv-FANN/ruv-swarm/npm

   # Install dependencies
   npm ci

   # Run security audit
   npm audit --audit-level moderate
   ```

2. **Build Production Artifacts**
   ```bash
   # Build WASM modules
   npm run build:wasm      # Standard WASM
   npm run build:wasm-simd  # SIMD optimized
   npm run build:wasm-opt   # Size optimized

   # Build JavaScript bundle
   npm run build
   ```

3. **Run Pre-deployment Checks**
   ```bash
   # Run tests
   npm run test:all

   # Run performance tests
   npm run test:performance

   # Run health checks
   node scripts/health-check.js
   ```

4. **Start the Application**
   ```bash
   # Development mode
   npm run dev

   # Production mode
   npm start

   # With PM2 (recommended for production)
   npm install -g pm2
   pm2 start src/index.js --name ruv-swarm
   ```

## Docker Deployment

### Using Docker Compose

```bash
# Start all services
docker-compose up -d

# Start with monitoring
docker-compose --profile monitoring up -d

# Start with logging
docker-compose --profile logging up -d

# View logs
docker-compose logs -f ruv-swarm

# Stop services
docker-compose down
```

### Using Docker Directly

```bash
# Build the image
docker build -t ruv-swarm:latest .

# Run the container
docker run -d \
  --name ruv-swarm \
  -p 3000:3000 \
  -v ruv-swarm-data:/app/data \
  ruv-swarm:latest

# Check health
curl http://localhost:3000/health
```

### Multi-Stage Deployment

```bash
# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t ruv-swarm:latest \
  --push \
  .
```

## Kubernetes Deployment

### Basic Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ruv-swarm
  labels:
    app: ruv-swarm
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ruv-swarm
  template:
    metadata:
      labels:
        app: ruv-swarm
    spec:
      containers:
      - name: ruv-swarm
        image: ghcr.io/ruvnet/ruv-fann/ruv-swarm:latest
        ports:
        - containerPort: 3000
        env:
        - name: NODE_ENV
          value: "production"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Service Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: ruv-swarm-service
spec:
  selector:
    app: ruv-swarm
  ports:
  - port: 3000
    targetPort: 3000
  type: ClusterIP
```

### Ingress Configuration

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ruv-swarm-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: ruv-swarm.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: ruv-swarm-service
            port:
              number: 3000
```

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ruv-swarm-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ruv-swarm
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Monitoring & Observability

### Health Checks

The application exposes health check endpoints:

```bash
# Application health
curl http://localhost:3000/health

# Detailed health check
node scripts/health-check.js --json

# Start health check server
node scripts/health-check.js --server --port 3001
```

### Metrics Collection

```bash
# Enable metrics collection
export METRICS_ENABLED=true
export METRICS_PORT=9090

# Start with metrics
npm start
```

### Logging

```bash
# Enable structured logging
export LOG_LEVEL=info
export LOG_FORMAT=json

# Log to file
export LOG_FILE=/var/log/ruv-swarm/app.log

# Start application
npm start
```

### Monitoring Stack

```bash
# Start with monitoring
docker-compose --profile monitoring up -d

# Access Grafana
open http://localhost:3002

# Access Prometheus
open http://localhost:9090
```

## Troubleshooting

### Common Issues

#### Build Failures

```bash
# Clear build cache
rm -rf node_modules/.cache
rm -rf target/

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Check system dependencies
node scripts/health-check.js
```

#### Runtime Issues

```bash
# Check application logs
docker-compose logs ruv-swarm

# Check health status
curl http://localhost:3000/health

# Run diagnostics
node scripts/health-check.js
```

#### Performance Issues

```bash
# Run performance tests
npm run test:performance

# Check memory usage
node scripts/health-check.js --json | jq '.metrics'

# Profile application
npm run profile
```

### Debug Mode

```bash
# Start in debug mode
NODE_ENV=development DEBUG=* npm start

# Enable verbose logging
export DEBUG=ruv-swarm:*

# Start with inspector
node --inspect src/index.js
```

## Rollback Procedures

### Quick Rollback

```bash
# Stop current deployment
docker-compose down

# Start previous version
docker-compose pull ruv-swarm:previous-version
docker-compose up -d
```

### Database Rollback

```bash
# Backup current database
cp data/database.db data/database.backup

# Restore from backup
cp data/database.backup data/database.db
```

### Full Rollback

```bash
# Stop all services
docker-compose down -v

# Remove all containers and volumes
docker system prune -f

# Start fresh deployment
docker-compose up -d --build
```

## Performance Optimization

### WASM Optimization

```bash
# Use optimized WASM build
npm run build:wasm-opt

# Enable SIMD if supported
npm run build:wasm-simd
```

### Memory Optimization

```bash
# Set memory limits
export NODE_OPTIONS="--max-old-space-size=2048"

# Enable garbage collection optimization
export NODE_OPTIONS="--optimize-for-size --memory-reducer"
```

### Caching

```bash
# Enable response caching
export CACHE_ENABLED=true
export CACHE_TTL=3600

# Enable WASM module caching
export WASM_CACHE_DIR=/tmp/wasm-cache
```

## Security Considerations

### Production Security

```bash
# Use secure headers
export SECURE_HEADERS=true

# Enable HTTPS
export HTTPS_ENABLED=true
export SSL_CERT_PATH=/path/to/cert.pem
export SSL_KEY_PATH=/path/to/key.pem

# Set security policies
export CSP="default-src 'self'"
export HSTS="max-age=31536000; includeSubDomains"
```

### Access Control

```bash
# Enable authentication
export AUTH_ENABLED=true
export JWT_SECRET=your-secret-key

# Set rate limiting
export RATE_LIMIT=1000
export RATE_WINDOW=3600
```

## Support

For deployment issues:

1. Check the [troubleshooting guide](#troubleshooting)
2. Run health checks: `node scripts/health-check.js`
3. Check logs: `docker-compose logs`
4. Review metrics in Grafana/Prometheus
5. Create an issue on GitHub with deployment logs

## Contributing

When contributing deployment improvements:

1. Test on multiple environments
2. Update this documentation
3. Add health checks for new features
4. Update monitoring configurations
5. Test rollback procedures