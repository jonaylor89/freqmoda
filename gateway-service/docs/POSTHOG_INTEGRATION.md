# PostHog Analytics Integration

This document describes how to integrate and configure PostHog analytics in the AI Audio Engineer gateway service to track user behavior and improve the product.

## Overview

PostHog has been integrated into all HTML templates to track user interactions, page views, and key events throughout the application. The integration is optional and can be enabled by configuring the PostHog settings in your configuration files.

## Configuration

### 1. Environment Setup

PostHog integration is configured through the application's configuration system. Add the following to your configuration files:

#### Base Configuration (`config/base.yml`)
```yaml
# PostHog Analytics (optional)
# Configure these values to enable user behavior tracking
# posthog:
#   api_key: "your-posthog-project-api-key"
#   host: "https://app.posthog.com"
```

#### Local Development (`config/local.yml`)
```yaml
# PostHog Analytics (optional)
# Uncomment and configure to enable analytics
# posthog:
#   api_key: "your-posthog-project-api-key"
#   host: "https://app.posthog.com"
```

#### Production (`config/production.yml`)
```yaml
posthog:
  api_key: "your-production-posthog-api-key"
  host: "https://app.posthog.com"
```

### 2. Environment Variables

You can also configure PostHog using environment variables:

```bash
export GATEWAY__POSTHOG__API_KEY="your-posthog-project-api-key"
export GATEWAY__POSTHOG__HOST="https://app.posthog.com"
```

### 3. Getting Your PostHog API Key

1. Sign up for a PostHog account at [https://posthog.com](https://posthog.com)
2. Create a new project or use an existing one
3. Go to Project Settings → API Keys
4. Copy your Project API Key (not the Personal API Key)
5. Use the API key in your configuration

## Tracked Events

The integration tracks the following events across all pages:

### Page Views
- `$pageview` - Standard PostHog page view event with custom properties

### Index Page (`/`)
- `info_modal_shown` - When the info modal is displayed to new users
- `info_modal_closed` - When the info modal is closed (with source: button_click, background_click, escape_key)
- `sample_selected` - When a user clicks to edit a sample
- `refresh_samples_clicked` - When the refresh button is clicked

### Sample Chat Page (`/sample/:id`)
- `back_to_samples_clicked` - When user navigates back to sample list
- `example_clicked` - When user clicks on example prompts
- `audio_play` / `audio_pause` - Audio playback controls
- `version_selected` - When user selects a different audio version
- `audio_downloaded` - When user downloads audio
- `sidebar_toggled` - Version history sidebar interactions
- `message_sent` - When user sends a chat message
- `message_failed` - When message sending fails
- `error_occurred` - Various error states

### Chat Page (`/chat`)
- `example_clicked` - When user clicks on example prompts
- `audio_play` / `audio_pause` - Audio playback controls
- `audio_downloaded` - When user downloads audio
- `message_sent` - When user sends a chat message
- `message_failed` - When message sending fails
- `error_occurred` - Various error states

## Event Properties

Events include contextual properties for better analysis:

### Common Properties
- `page` - Current page identifier
- `session_id` - User session ID
- `sample_key` - Sample identifier (where applicable)
- `sample_title` - Sample title (where applicable)

### Message Events
- `message_length` - Length of the user's message
- `has_audio_keywords` - Boolean indicating if message contains audio processing terms

### Audio Events
- `version_index` - Which version of audio was interacted with
- `version_description` - Description of the audio version
- `location` - Where the audio player is located (inline, main, etc.)

### Error Events
- `error_type` - Type of error that occurred
- `error` - Error message (for failed operations)

## Privacy and GDPR Compliance

PostHog is configured with `person_profiles: "identified_only"` which means:
- Anonymous users are not tracked with persistent profiles
- Only users who explicitly identify themselves get persistent profiles
- This helps with GDPR compliance for EU users

You may want to add additional privacy controls:
- Cookie consent banners
- Opt-out mechanisms
- Data retention policies

## Analytics Dashboard

Once PostHog is configured, you can access your analytics dashboard at:
- [https://app.posthog.com](https://app.posthog.com) (for PostHog Cloud)
- Your self-hosted PostHog instance URL

### Useful Dashboards to Create

1. **User Journey Analysis**
   - Track flow from landing page → sample selection → chat interaction
   - Identify drop-off points in the user journey

2. **Feature Usage**
   - Audio playback frequency
   - Download rates
   - Example prompt usage
   - Version history interactions

3. **Error Monitoring**
   - Track error rates and types
   - Monitor service availability issues
   - Identify common user problems

4. **Conversion Funnel**
   - Landing page visits → sample interactions → successful audio processing

## Development and Testing

### Local Development
- PostHog integration is optional - the app works without it configured
- Use a separate PostHog project for development to avoid mixing development data with production analytics

### Testing Events
You can test event tracking by:
1. Configuring PostHog in your local environment
2. Interacting with the application
3. Checking the PostHog dashboard for events
4. Using browser developer tools to see PostHog network requests

## Troubleshooting

### Events Not Appearing
1. Check that `posthog_api_key` and `posthog_host` are correctly configured
2. Verify the API key is correct in your PostHog project settings
3. Check browser console for JavaScript errors
4. Ensure the PostHog script is loading (check Network tab in dev tools)

### Performance Considerations
- PostHog scripts are loaded asynchronously and won't block page rendering
- Events are batched and sent efficiently
- Consider using PostHog's session recording features carefully as they can impact performance

### Configuration Issues
- Ensure configuration follows the exact YAML structure
- Check that environment variables use the correct prefix (`GATEWAY__POSTHOG__`)
- Verify the application is reading the configuration correctly by checking logs

## Advanced Configuration

### Custom Event Properties
You can extend the tracking by modifying the JavaScript functions in the templates to include additional properties relevant to your use case.

### Feature Flags
PostHog supports feature flags which can be used to:
- A/B test different UI variations
- Gradually roll out new features
- Enable/disable features for specific user segments

### Cohorts and User Segmentation
Use PostHog's cohort features to:
- Analyze power users vs. casual users
- Track user retention over time
- Segment users by behavior patterns

## Security Notes

- Store API keys securely (use environment variables in production)
- The PostHog API key is included in client-side JavaScript, which is normal and expected
- For sensitive applications, consider using PostHog's EU hosting or self-hosted options
- Regularly rotate API keys as part of security best practices

## Resources

- [PostHog Documentation](https://posthog.com/docs)
- [PostHog JavaScript Library](https://posthog.com/docs/libraries/js)
- [PostHog API Reference](https://posthog.com/docs/api)
- [Privacy and GDPR Guide](https://posthog.com/docs/privacy)