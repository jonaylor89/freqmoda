#!/bin/bash

echo "🧪 Testing Web Demo Deployment Setup"
echo "=========================================="

# Check if we're in the right directory
if [ ! -d "web-demo" ]; then
    echo "❌ Please run this script from the freqmoda root directory"
    exit 1
fi

cd web-demo

# Test Rust build
echo "📦 Testing Rust build..."
if cargo build --release; then
    echo "✅ Rust build successful"
else
    echo "❌ Rust build failed"
    exit 1
fi

# Check for required config files
echo "🔧 Checking configuration files..."
if [ -f "config/production.yml" ]; then
    echo "✅ Production config exists"
    
    # Check if it contains required fields
    if grep -q "database:" config/production.yml && grep -q "claude:" config/production.yml; then
        echo "✅ Production config has required sections"
    else
        echo "⚠️  Production config may be missing required sections"
    fi
else
    echo "❌ config/production.yml missing - this is required for deployment"
    echo "💡 Copy from config.template.yml and update with your values"
    exit 1
fi

# Check .gcloudignore exists and doesn't exclude config
echo "☁️  Checking .gcloudignore configuration..."
if [ -f ".gcloudignore" ]; then
    if grep -q "^config/" .gcloudignore; then
        echo "❌ .gcloudignore excludes config/ - this will break deployment"
    else
        echo "✅ .gcloudignore allows config files"
    fi
else
    echo "✅ No .gcloudignore found (config files will be included)"
fi

# Test migrations directory
echo "🗄️ Testing database migrations..."
if [ -d "migrations" ] && [ "$(ls -A migrations)" ]; then
    echo "✅ Database migrations found"
else
    echo "❌ No database migrations found"
fi

# Test templates directory
echo "🎨 Testing templates..."
if [ -d "templates" ] && [ "$(ls -A templates)" ]; then
    echo "✅ Templates directory found"
else
    echo "❌ No templates found"
fi

# Test server startup (background)
echo "🚀 Testing server startup..."
timeout 15s cargo run &
SERVER_PID=$!
sleep 8

# Test health endpoint
echo "💚 Testing health endpoint..."
if curl -f http://localhost:9000/health >/dev/null 2>&1; then
    echo "✅ Health endpoint responding"
else
    echo "❌ Health endpoint not responding"
fi

# Test web interface
echo "🌐 Testing web interface..."
if curl -f http://localhost:9000/ >/dev/null 2>&1; then
    echo "✅ Web interface responding"
else
    echo "❌ Web interface not responding"
fi

# Stop server
kill $SERVER_PID 2>/dev/null
sleep 2

# Test CLI build
echo "📱 Testing CLI tool build..."
if cargo build --bin chat-cli; then
    echo "✅ CLI tool build successful"
else
    echo "❌ CLI tool build failed"
fi

# Test Docker build (if Docker is available)
if command -v docker &> /dev/null; then
    echo "🐳 Testing Docker build..."
    if docker build -t web-demo-test . > /tmp/docker-build.log 2>&1; then
        echo "✅ Docker build successful"
        docker rmi web-demo-test >/dev/null 2>&1
    else
        echo "❌ Docker build failed"
        echo "🔍 Last 10 lines of build output:"
        tail -10 /tmp/docker-build.log
    fi
    rm -f /tmp/docker-build.log
else
    echo "⚠️  Docker not available, skipping Docker test"
fi

cd ..

echo ""
echo "🎉 Web Demo deployment test complete!"
echo ""
echo "Next steps:"
echo "1. Ensure your config/production.yml has all required values:"
echo "   - Supabase database connection"
echo "   - Upstash Redis URL" 
echo "   - Claude API key"
echo "   - Streaming engine URL"
echo "2. Deploy to Cloud Run:"
echo "   ./scripts/deploy-web-demo.sh YOUR_PROJECT_ID us-central1"
echo "3. Test deployed service:"
echo "   curl https://your-service-url/health"
echo "4. Access web interface:"
echo "   open https://your-service-url"
