# PostHog Proxy Setup

This document explains the PostHog reverse proxy implementation in the gateway service.

## Overview

The gateway service includes a reverse proxy for PostHog analytics to:
- Avoid ad blockers that might block PostHog requests
- Use your own domain for analytics requests
- Improve reliability and performance
- Maintain user privacy by not exposing PostHog directly

## Implementation

### Proxy Routes

The following routes are configured to proxy PostHog requests:

| Local Route | PostHog Endpoint | Purpose |
|-------------|------------------|---------|
| `/internal/metrics/assets/*` | `/static/*` | PostHog JavaScript library and assets |
| `/internal/metrics/collect` | `/capture/` | Event capture endpoint |
| `/internal/metrics/batch` | `/batch/` | Batch event submission |
| `/internal/metrics/decide` | `/decide/` | Feature flags and configuration |

### Route Naming Strategy

The routes use obscure naming (`/internal/metrics/`) instead of obvious PostHog-related paths to:
- Avoid detection by ad blockers
- Make the analytics less obvious to end users
- Prevent automated blocking of analytics requests

### Configuration

PostHog configuration is handled through the existing config system:

```toml
[posthog]
api_key = "your-posthog-api-key"
host = "https://app.posthog.com"  # or your PostHog instance URL
```

### Client Integration

In HTML templates, PostHog is configured to use the local proxy:

```javascript
posthog.init("your-api-key", {
    api_host: "/internal/metrics",
    person_profiles: "identified_only",
});
```

The PostHog JavaScript library is loaded from:
```html
<script src="/internal/metrics/assets/array.js"></script>
```

## How It Works

1. **Static Assets**: When the client requests `/internal/metrics/assets/array.js`, the proxy fetches it from PostHog's CDN and serves it
2. **Event Capture**: Analytics events sent to `/internal/metrics/collect` are forwarded to PostHog's `/capture/` endpoint
3. **Header Forwarding**: Relevant headers are forwarded while filtering out hop-by-hop headers
4. **CORS Support**: Proper CORS headers are added for browser compatibility

## Benefits

### Ad Blocker Resistance
- Uses generic internal API paths instead of obvious analytics endpoints
- Serves from your own domain rather than third-party analytics domains

### Performance
- Reduces external dependencies
- Can implement caching if needed in the future
- Single domain for all requests

### Privacy
- PostHog URLs are not exposed to the client
- All analytics requests appear to be internal API calls

## Security Considerations

### Header Filtering
The proxy filters out hop-by-hop headers to prevent security issues:
- `connection`
- `keep-alive` 
- `proxy-authenticate`
- `proxy-authorization`
- `te`
- `trailers`
- `transfer-encoding`
- `upgrade`
- `host`

### CORS Headers
Appropriate CORS headers are added:
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, Authorization`

## Error Handling

The proxy includes comprehensive error handling:
- Returns `503 Service Unavailable` if PostHog is not configured
- Returns `502 Bad Gateway` if the upstream request fails
- Logs errors for debugging while maintaining service availability

## Testing

Basic tests are included to verify:
- Error handling for invalid upstream URLs
- CORS header configuration
- Response status codes

## Monitoring

The proxy logs all requests at the debug level:
```
Proxying metrics collect request to: https://app.posthog.com/capture/
```

This allows for monitoring and debugging without exposing sensitive information.

## Deployment Notes

1. Ensure PostHog configuration is properly set in your environment
2. The proxy requires network access to PostHog's servers
3. Consider implementing rate limiting if needed
4. Monitor proxy performance and error rates

## Future Enhancements

Potential improvements:
- Response caching for static assets
- Request batching and queuing
- Analytics data enrichment
- Custom event validation
- Request rate limiting per client