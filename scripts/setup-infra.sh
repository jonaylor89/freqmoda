#!/bin/bash

set -e

PROJECT_ID=${1:-"your-project-id"}
REGION=${2:-"us-central1"}

echo "🏗️  Setting up minimal infrastructure for streaming engine"
echo "Project: $PROJECT_ID"
echo "Region: $REGION"
echo ""

# Enable only the required APIs
echo "📋 Enabling minimal required APIs..."
gcloud services enable \
  run.googleapis.com \
  storage.googleapis.com \
  --project=$PROJECT_ID

# Create the ONLY external resource needed: Storage bucket
BUCKET_NAME="${PROJECT_ID}-test-audio"
echo "📦 Creating minimal storage bucket: $BUCKET_NAME"

# Create bucket with cost-optimized settings
gsutil mb \
  -p $PROJECT_ID \
  -c STANDARD \
  -l $REGION \
  gs://$BUCKET_NAME/ 2>/dev/null || echo "✅ Bucket already exists"


# Set bucket to auto-delete temporary files
echo "🗄️  Setting bucket permissions..."
gsutil iam ch allUsers:objectViewer gs://$BUCKET_NAME/

echo ""
echo "✅ Infrastructure setup complete!"
echo ""
echo "📊 Resources created:"
echo "  • Cloud Storage bucket: gs://$BUCKET_NAME"
echo "  • Cache: Ephemeral filesystem (no external cache)"
echo ""
echo "🚀 Ready to deploy with:"
echo "  ./scripts/deploy-cloud-run.sh $PROJECT_ID $REGION"