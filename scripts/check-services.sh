#!/bin/bash

PROJECT_ID=${1:-"your-project-id"}
REGION=${2:-"us-central1"}

echo "🔍 Checking FreqModa Services Status"
echo "Project: $PROJECT_ID"
echo "Region: $REGION"
echo "=================================="

# Set project
gcloud config set project $PROJECT_ID >/dev/null 2>&1

# Check Streaming Engine
echo "🎵 Streaming Engine:"
STREAMING_URL=$(gcloud run services describe streaming-engine --region=$REGION --format="value(status.url)" --project=$PROJECT_ID 2>/dev/null || echo "")
if [ ! -z "$STREAMING_URL" ]; then
    echo "   ✅ Deployed: $STREAMING_URL"
    if curl -f "$STREAMING_URL/health" >/dev/null 2>&1; then
        echo "   ✅ Health check: OK"
    else
        echo "   ❌ Health check: FAILED"
    fi
else
    echo "   ❌ Not deployed"
fi

echo ""

# Check Gateway Service
echo "🤖 Gateway Service:"
GATEWAY_URL=$(gcloud run services describe gateway-service --region=$REGION --format="value(status.url)" --project=$PROJECT_ID 2>/dev/null || echo "")
if [ ! -z "$GATEWAY_URL" ]; then
    echo "   ✅ Deployed: $GATEWAY_URL"
    if curl -f "$GATEWAY_URL/health" >/dev/null 2>&1; then
        echo "   ✅ Health check: OK"
    else
        echo "   ❌ Health check: FAILED"
    fi
else
    echo "   ❌ Not deployed"
fi

echo ""

# Check Database (External - Supabase)
echo "🗄️ Database:"
echo "   🌐 Using external Supabase PostgreSQL"
echo "   ℹ️  Cannot check external database status from here"
echo "   💡 Verify your Supabase connection manually"

echo ""

# Check Redis (External - Upstash)
echo "🔴 Redis:"
echo "   🌐 Using external Upstash Redis"
echo "   ℹ️  Cannot check external Redis status from here"
echo "   💡 Verify your Upstash connection manually"

echo ""

# Integration Test
if [ ! -z "$GATEWAY_URL" ] && [ ! -z "$STREAMING_URL" ]; then
    echo "🧪 Integration Test:"
    echo "   Testing chat endpoint..."

    RESPONSE=$(curl -s -X POST "$GATEWAY_URL/api/chat" \
        -H "Content-Type: application/json" \
        -d '{"message": "Hello"}' 2>/dev/null || echo "")

    if [ ! -z "$RESPONSE" ] && echo "$RESPONSE" | grep -q "message"; then
        echo "   ✅ Chat API: Working"
    else
        echo "   ❌ Chat API: Failed"
    fi
fi

echo ""
echo "📋 Quick Access URLs:"
if [ ! -z "$STREAMING_URL" ]; then
    echo "Streaming Engine: $STREAMING_URL"
fi
if [ ! -z "$GATEWAY_URL" ]; then
    echo "Gateway Service:  $GATEWAY_URL"
    echo "Web Interface:    $GATEWAY_URL"
fi

echo ""
echo "📊 Monitoring:"
echo "Cloud Run Console: https://console.cloud.google.com/run?project=$PROJECT_ID"
echo "Supabase Console:  https://app.supabase.com/"
echo "Upstash Console:   https://console.upstash.com/"
