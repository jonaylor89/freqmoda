#!/bin/bash

set -e

PROJECT_ID=${1:-"your-project-id"}
REGION=${2:-"us-central1"}
SERVICE_NAME="web-ui"

echo "🚀 Deploying Web UI to Google Cloud Run"
echo "Project: $PROJECT_ID"
echo "Region: $REGION"
echo ""

# Set project
gcloud config set project $PROJECT_ID

# Enable required APIs
echo "🔧 Enabling required APIs..."
gcloud services enable cloudbuild.googleapis.com
gcloud services enable run.googleapis.com
gcloud services enable containerregistry.googleapis.com

# Check if streaming engine is deployed
echo "🔍 Checking for streaming engine deployment..."
STREAMING_ENGINE_URL=$(gcloud run services describe streaming-engine --region=$REGION --format="value(status.url)" --project=$PROJECT_ID 2>/dev/null || echo "")

if [ -z "$STREAMING_ENGINE_URL" ]; then
    echo "⚠️  Warning: Streaming engine not found. Deploy it first with:"
    echo "   ./scripts/deploy-streaming-engine.sh $PROJECT_ID $REGION"
    echo ""
    echo "Continuing with deployment..."
else
    echo "✅ Found streaming engine at: $STREAMING_ENGINE_URL"
fi

# Prepare Dockerfile for deployment
echo "📦 Preparing Dockerfile..."
cp Dockerfile.web-ui Dockerfile

# Deploy to Cloud Run
echo "🚀 Deploying to Cloud Run..."
gcloud run deploy $SERVICE_NAME \
    --source . \
    --platform managed \
    --region $REGION \
    --allow-unauthenticated \
    --memory 1Gi \
    --cpu 1 \
    --timeout 300 \
    --max-instances 10 \
    --min-instances 0 \
    --cpu-throttling \
    --execution-environment gen2 \
    --set-env-vars="APP_ENVIRONMENT=production" \
    --set-env-vars="WEB_UI_SERVER__HOST=0.0.0.0" \
    --port=9000 \
    --project=$PROJECT_ID

# Clean up temporary Dockerfile
rm -f Dockerfile

# Get the service URL
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region=$REGION --format="value(status.url)" --project=$PROJECT_ID)

echo ""
echo "✅ Deployment complete!"
echo "🌐 Service URL: $SERVICE_URL"
echo ""
echo "🧪 Test your deployment:"
echo "# Health check"
echo "curl $SERVICE_URL/health"
echo ""
echo "# Web interface"
echo "open $SERVICE_URL"
echo ""
echo "# Chat API test"
echo "curl -X POST $SERVICE_URL/api/chat \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"message\": \"Hello, can you help me process audio?\"}'"
echo ""
echo "🎛️ Monitor your deployment:"
echo "https://console.cloud.google.com/run/detail/$REGION/$SERVICE_NAME/metrics?project=$PROJECT_ID"
echo ""
echo "📊 View logs:"
echo "gcloud run services logs tail $SERVICE_NAME --region=$REGION --project=$PROJECT_ID"
