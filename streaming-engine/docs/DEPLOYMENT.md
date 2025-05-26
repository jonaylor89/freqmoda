# Deployment Guide

## Google Cloud Run Deployment

### Prerequisites
- Google Cloud Project with billing enabled
- Cloud Run API enabled
- Cloud Build API enabled (for automatic deployment)
- Docker installed locally (for manual deployment)

### Option 1: Automatic Deployment with GitHub Integration

1. **Connect GitHub Repository**
   ```bash
   # Enable required APIs
   gcloud services enable cloudbuild.googleapis.com
   gcloud services enable run.googleapis.com
   gcloud services enable containerregistry.googleapis.com
   ```

2. **Set up GitHub Integration in Cloud Console**
   - Go to Cloud Build > Triggers
   - Connect your GitHub repository
   - Create a trigger that builds on push to `main` branch
   - Point to the `cloudbuild.yaml` file

3. **Update Configuration**
   - Edit `config/production.yml` and set your Cloud Run URL
   - Update `cloudbuild.yaml` with your preferred region and settings
   - Push to main branch to trigger deployment

### Option 2: Manual Deployment

1. **Build and Deploy**
   ```bash
   # Set your project
   gcloud config set project YOUR_PROJECT_ID
   
   # Build and deploy
   gcloud run deploy streaming-engine \
     --source . \
     --region us-central1 \
     --allow-unauthenticated \
     --memory 2Gi \
     --cpu 2 \
     --max-instances 10 \
     --timeout 900 \
     --port 8080 \
     --set-env-vars APP_ENVIRONMENT=production
   ```

2. **Get Service URL**
   ```bash
   gcloud run services describe streaming-engine --region=us-central1 --format="value(status.url)"
   ```

### Cloud Run Configuration

**Recommended Settings:**
- **Memory**: 2Gi (for FFmpeg processing)
- **CPU**: 2 (for concurrent audio processing)
- **Timeout**: 900s (15min for large audio files)
- **Max Instances**: 10 (adjust based on usage)
- **Concurrency**: 80 (default, adjust if needed)

**Environment Variables:**
- `APP_ENVIRONMENT=production`
- `APP_APPLICATION__HOST=0.0.0.0`
- Optional: `PORT` (Cloud Run sets this automatically)

### Storage Configuration

For production, consider using Cloud Storage instead of ephemeral storage:

1. **Create Cloud Storage Bucket**
   ```bash
   gsutil mb gs://your-streaming-engine-bucket
   ```

2. **Update production.yml**
   ```yaml
   storage:
     client:
       GCS:
         bucket: "your-streaming-engine-bucket"
         credentials: "path-to-service-account-key"
   ```

3. **Set up Service Account**
   ```bash
   # Create service account
   gcloud iam service-accounts create streaming-engine-storage
   
   # Grant storage permissions
   gcloud projects add-iam-policy-binding YOUR_PROJECT_ID \
     --member="serviceAccount:streaming-engine-storage@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
     --role="roles/storage.admin"
   
   # Create and download key
   gcloud iam service-accounts keys create key.json \
     --iam-account=streaming-engine-storage@YOUR_PROJECT_ID.iam.gserviceaccount.com
   ```

## MCP Server Distribution

### Publishing to npm

1. **Login to npm**
   ```bash
   cd mcp-server
   npm login
   ```

2. **Publish**
   ```bash
   npm publish --access public
   ```

3. **Users can then run**
   ```bash
   npx @streaming-engine/mcp-server --server=https://your-app.run.app
   ```

### Alternative Distribution

Create a simple installation script:
```bash
# install-mcp.sh
curl -o streaming-engine-mcp.js https://raw.githubusercontent.com/jonaylor89/freqmoda/main/streaming-engine/main/mcp-server/index.js
node streaming-engine-mcp.js
```

## Security Considerations

### Production Security
1. **Enable URL Signing**
   - Set `APP_APPLICATION__HMAC_SECRET` environment variable
   - Use signed URLs instead of `/unsafe/` endpoints

2. **Rate Limiting**
   - Configure rate limits in production.yml
   - Consider Cloud Armor for additional protection

3. **CORS Configuration**
   - Configure allowed origins for web clients
   - Set appropriate CORS headers

### MCP Server Security
- The MCP server connects to your streaming engine instance
- Only use trusted streaming engine server URLs
- Consider authentication for production MCP deployments

## Monitoring and Logging

### Cloud Run Monitoring
- Built-in metrics in Cloud Console
- Set up alerts for high CPU/memory usage
- Monitor request latency and error rates

### Application Logs
```bash
# View logs
gcloud run services logs read streaming-engine --region=us-central1

# Follow logs
gcloud run services logs tail streaming-engine --region=us-central1
```

### Health Checks
The `/health` endpoint provides service status:
```bash
curl https://your-app.run.app/health
```

## Troubleshooting

### Common Issues

1. **SSL Library Errors** (`libssl.so.3: cannot open shared object file`)
   - Fixed by using `debian:bookworm-slim` runtime image
   - Bookworm includes OpenSSL 3.x required by newer Rust builds

2. **Memory Limits**
   - Increase memory allocation for large audio files
   - Optimize caching settings in production.yml

3. **Timeout Issues**
   - Increase timeout for long audio processing
   - Consider async processing for very large files

4. **Storage Issues**
   - Ensure Cloud Storage permissions are correct
   - Check bucket exists and is accessible

5. **MCP Connection Issues**
   - Verify streaming engine server is accessible
   - Check network connectivity
   - Validate server URL format

### Debug Commands
```bash
# Test health endpoint
curl https://your-app.run.app/health

# Test API schema
curl https://your-app.run.app/api-schema

# Check service status
gcloud run services describe streaming-engine --region=us-central1

# View recent logs
gcloud run services logs read streaming-engine --region=us-central1 --limit=50
```