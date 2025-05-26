# Gateway Service Deployment

Quick deployment guide for the FreqModa Gateway Service to Google Cloud Run using existing config files.

## Prerequisites

- Google Cloud Project with billing enabled
- gcloud CLI installed and configured
- Existing `config/production.yml` with all your secrets (Supabase, Upstash, Claude API key)
- Docker installed locally (optional)

## Configuration

Your `config/production.yml` already contains all necessary configuration including:
- Supabase PostgreSQL connection details
- Upstash Redis URL
- Claude API key
- Streaming engine URL

Since `config/` is in `.gitignore`, the deployment uses `.gcloudignore` to ensure config files are included in the Cloud Build context.

## Quick Deploy

### 1. Simple Deployment

```bash
# Clone the repository (if needed)
git clone <your-repo-url>
cd freqmoda

# Set your project ID
export PROJECT_ID="your-google-cloud-project-id"

# Deploy (uses existing config/production.yml)
./scripts/deploy-gateway-service.sh $PROJECT_ID us-central1
```

### 2. Test Deployment

```bash
# The script will output your service URL
export SERVICE_URL="https://gateway-service-xxxxx.run.app"

# Test health endpoint
curl $SERVICE_URL/health

# Test web interface
open $SERVICE_URL

# Test chat API
curl -X POST $SERVICE_URL/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello, can you help me process audio?"}'
```

## How Config Files Work

1. **Local Development**: `config/` is in `.gitignore` (not committed to Git)
2. **Cloud Build**: `.gcloudignore` ensures config files ARE included in build context
3. **Docker Build**: Dockerfile copies config files from build context
4. **Runtime**: Service reads `config/production.yml` when `APP_ENVIRONMENT=production`

## Manual Setup (Alternative)

If you prefer step-by-step deployment:

```bash
cd gateway-service

gcloud run deploy gateway-service \
  --source . \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 1Gi \
  --cpu 1 \
  --max-instances 10 \
  --timeout 300 \
  --port 9000 \
  --set-env-vars APP_ENVIRONMENT=production \
  --set-env-vars GATEWAY_SERVER__HOST=0.0.0.0 \
  --project $PROJECT_ID
```

## Configuration Details

The service uses these settings from your `config/production.yml`:
- **Server**: Host and port configuration
- **Database**: Supabase PostgreSQL connection
- **Redis**: Upstash Redis URL
- **Claude**: API key for AI processing
- **Streaming Engine**: URL for audio processing

## Dependencies

The gateway service requires:
- **Streaming Engine**: Should be deployed first
- **Supabase**: PostgreSQL database (configured in your config file)
- **Upstash**: Redis instance (configured in your config file)
- **Claude API**: Valid API key with credits (configured in your config file)

## Monitoring

### View Logs
```bash
gcloud run services logs tail gateway-service --region=us-central1
```

### Monitor Performance
```bash
# View service details
gcloud run services describe gateway-service --region=us-central1

# Open Cloud Console monitoring
open "https://console.cloud.google.com/run/detail/us-central1/gateway-service/metrics?project=$PROJECT_ID"
```

## Scaling

### Update Resources
```bash
gcloud run services update gateway-service \
  --memory 2Gi \
  --cpu 2 \
  --max-instances 20 \
  --region=us-central1
```

### Auto-scaling Settings
- **Min Instances**: 0 (scale to zero)
- **Max Instances**: 10 (adjust based on usage)
- **Concurrency**: 80 per instance
- **CPU Allocation**: Always allocated during request processing

## Troubleshooting

### Common Issues

**Service Won't Start**
```bash
# Check recent logs for errors
gcloud run services logs read gateway-service --limit=50

# Check service status
gcloud run services describe gateway-service --region=us-central1
```

**Config Files Missing**
```bash
# Verify config files exist locally
ls -la gateway-service/config/

# Check .gcloudignore doesn't exclude config/
cat gateway-service/.gcloudignore | grep -v "^#" | grep config
```

**Database Connection Failed**
- Verify Supabase credentials in `config/production.yml`
- Check Supabase dashboard for connection limits
- Test connection manually with psql

**Claude API Issues**
- Verify API key in `config/production.yml`
- Check Claude API credits and rate limits
- Test API key with curl request

### Debug Commands

```bash
# Test local build
cd gateway-service && cargo build --release

# Test Docker build locally
docker build -t gateway-test gateway-service/

# Check configuration loading
cargo run --bin gateway-service 2>&1 | grep -i config

# Test streaming engine connection
curl https://your-streaming-engine-url/health
```

## Security

- All secrets stored in config files (not in environment variables)
- Config files excluded from Git but included in builds
- HTTPS enforced by Cloud Run
- External services (Supabase/Upstash) handle their own security

## Cost Optimization

- Scale-to-zero when not in use
- Right-size memory and CPU based on usage
- Monitor with billing alerts
- Use Supabase/Upstash free tiers for development

## Updates

To update the service:
1. Update your local config files if needed
2. Run the deployment script again
3. New revision will be deployed automatically

```bash
./scripts/deploy-gateway-service.sh $PROJECT_ID us-central1
```