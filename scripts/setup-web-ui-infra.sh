#!/bin/bash

set -e

PROJECT_ID=${1:-"your-project-id"}
REGION=${2:-"us-central1"}

echo "🏗️  Setting up Web UI Infrastructure (External Services)"
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
gcloud services enable secretmanager.googleapis.com

# Create service account
echo "🔐 Creating service account..."
SERVICE_ACCOUNT_NAME="web-ui"
SERVICE_ACCOUNT="$SERVICE_ACCOUNT_NAME@$PROJECT_ID.iam.gserviceaccount.com"

if ! gcloud iam service-accounts describe $SERVICE_ACCOUNT --project=$PROJECT_ID >/dev/null 2>&1; then
    gcloud iam service-accounts create $SERVICE_ACCOUNT_NAME \
        --display-name="Web UI Service Account" \
        --description="Service account for Web UI Cloud Run deployment" \
        --project=$PROJECT_ID
    echo "✅ Service account created"
else
    echo "✅ Service account already exists"
fi

# Grant necessary IAM roles
echo "🔑 Granting IAM roles..."
gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:$SERVICE_ACCOUNT" \
    --role="roles/secretmanager.secretAccessor"

echo ""
echo "✅ Infrastructure setup complete!"
echo ""
echo "📋 Configuration Details:"
echo "Service Account: $SERVICE_ACCOUNT"
echo ""
echo "🔗 External Services Required:"
echo "1. Supabase PostgreSQL database"
echo "2. Upstash Redis instance"
echo "3. Claude API key from Anthropic"
echo "4. Deployed streaming engine instance"
echo ""
echo "🚀 Ready for deployment! Run:"
echo "./scripts/deploy-web-ui.sh $PROJECT_ID $REGION"
