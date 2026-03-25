# Streaming Engine GTM Plan

## TL;DR

- If you ignore `web-demo`, the streaming engine is not a SaaS app yet; it is an audio transformation infrastructure product.
- The fastest path to revenue is a developer/infra sale, not a consumer audio editor: "hosted audio-processing API + MCP server for LLM workflows."
- Best first ICP: AI product teams building voice/media features, then audio-tool vendors that want embedded processing, then advanced creator/dev hybrids.
- Best monetization order: API contract or pilot first, then usage-based with minimum commits, then embedded/white-label deals.
- Biggest blockers before charging self-serve: customer auth, API keys, metering, signed access instead of `/unsafe`, and production-safe ingestion controls.

Assumptions: this is scoped only to `streaming-engine/`; `web-demo` is excluded entirely.

## 1. ICP

### 1. AI product teams adding audio actions to agents

- Who: 3-50 person startups shipping voice agents, transcription workflows, media copilots, or LLM tools.
- Pain: they need programmable audio transforms without building and operating FFmpeg orchestration themselves.
- WTP: `$500-$3k+/mo` in pilot/contract form if the engine reduces time-to-market.
- Where to find them: MCP/server builders, Claude/agent-tooling communities, GitHub, and founder-led outbound into AI startups.

Why this fits the repo:

- The engine already exposes direct processing, metadata, params preview, OpenAPI, and an MCP server. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L166), [index.js](/Users/johannes/Repos/freqmoda/streaming-engine/mcp-server/index.js#L31), and [package.json](/Users/johannes/Repos/freqmoda/streaming-engine/mcp-server/package.json#L1).

### 2. Audio software vendors and workflow platforms

- Who: small-to-mid audio SaaS vendors, transcription/post-production vendors, broadcast tooling companies.
- Pain: they want to embed processing features without turning into an audio infrastructure company.
- WTP: `$10k-$50k+` pilot or annual contract if you can offer hosted API or white-label access.
- Where to find them: direct outbound, integration partnerships, audio software directories, and industry communities like KVR.

Why this fits the repo:

- The storage abstraction and backend options already point toward embeddable infrastructure rather than end-user product UX. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L58) and [backend.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/storage/backend.rs#L1).

### 3. Advanced creators who automate their stack

- Who: technical podcasters, sample creators, and power users who prefer APIs, scripts, or LLM workflows over GUI editing.
- Pain: repeated audio transforms are tedious in traditional editors.
- WTP: `$29-$199/mo`, but only after you make the engine safer and easier to self-serve.
- Where to find them: Product Hunt, GitHub, KVR, and maker communities.

## 2. Fastest Path to Revenue (90-Day Sprint)

Recommendation: sell the streaming engine as a hosted API plus MCP pilot to AI product teams.

This is the fastest path because the engine already has:

- HTTP endpoints for processing, params preview, metadata, health, metrics, and schema. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L166).
- An MCP server that exposes `process_audio`, `preview_audio_params`, `get_audio_metadata`, and health checks. See [index.js](/Users/johannes/Repos/freqmoda/streaming-engine/mcp-server/index.js#L31).
- Multiple storage backends and deploy scripts for Cloud Run. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L58), [deploy-streaming-engine.sh](/Users/johannes/Repos/freqmoda/scripts/deploy-streaming-engine.sh#L24), and [Dockerfile.streaming](/Users/johannes/Repos/freqmoda/Dockerfile.streaming#L1).

Single fastest offer:

- "We'll give your team a hosted audio-processing endpoint and MCP server for your agent workflow. You send file URLs or storage keys; we return processed audio deterministically."

90-day motion:

1. Weeks 1-2: private pilot offer.
   - Package one hosted endpoint.
   - Gate access manually.
   - Onboard 3-5 design partners.
2. Weeks 3-6: add only revenue-critical controls.
   - customer API keys
   - request logging
   - per-customer quotas
   - signed access instead of public `/unsafe`
3. Weeks 7-12: convert first pilot into recurring contract.
   - minimum monthly commit
   - support SLA
   - optional private MCP endpoint for the customer's internal agent stack

Concierge MVP:

- Manual onboarding.
- Customer provides remote URLs or bucket paths.
- You hand them an endpoint, example cURL, and MCP config.
- You monitor jobs manually and tune deployment and limits.

## 3. Monetization Models

### 1. API access tiers

- Fit: `9/10`
- Pros: strongest match to the current repo shape; buyers can integrate immediately; easy story for AI product teams.
- Cons: needs auth, metering, abuse controls, and customer support tooling.
- Recommendation: primary model.

### 2. White-label / embedded licensing

- Fit: `8/10`
- Pros: highest ACV; strong fit for infrastructure buyers; adjacent companies already sell API and white-label audio infrastructure.
- Cons: slower sales cycle; more enterprise/security demands.
- Recommendation: second model after first 2-3 successful pilots.

### 3. Usage-based per minute of audio processed

- Fit: `8/10`
- Pros: natural fit for infrastructure costs; familiar in audio tooling.
- Cons: you need robust duration metering and minimum-charge logic.
- Recommendation: best billing primitive under the hood, even if contracts start as fixed-fee pilots.

### 4. Credits model

- Fit: `6/10`
- Pros: simple for small devs and power users; works with bursty demand.
- Cons: weaker for B2B infra buyers; lower predictability.
- Recommendation: useful later for self-serve.

### 5. Seat-based SaaS

- Fit: `2/10`
- Pros: predictable.
- Cons: wrong product shape if you are ignoring `web-demo`; the engine has no collaborative UI or seat-based usage concept.
- Recommendation: do not lead with this.

## 4. MVP Feature Gaps

### P0: customer authentication and API keys

- There is no inbound customer auth model on the engine today.
- The only protection in the request path is hash verification, but requests explicitly bypass it when the path starts with `unsafe`. See [middleware.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/middleware.rs#L135).
- The current MCP server also targets `/unsafe/...` directly. See [index.js](/Users/johannes/Repos/freqmoda/streaming-engine/mcp-server/index.js#L210).

What's missing:

- API keys
- customer identity
- key rotation/revocation
- per-key scopes
- per-customer usage attribution

### P0: usage metering

- There is metrics exposure, but no billing-grade usage ledger tied to customers. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L166).

You need:

- input duration
- output duration
- bytes processed
- storage reads/writes
- success/failure
- customer/key attribution

### P0: safe production access model

- The repo is still demo/open-server oriented:
  - permissive CORS in the engine. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L198).
  - unauthenticated Cloud Run deployment script. See [deploy-streaming-engine.sh](/Users/johannes/Repos/freqmoda/scripts/deploy-streaming-engine.sh#L26).
  - public `/unsafe` processing path. See [middleware.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/middleware.rs#L154).
- Before charging, replace this with signed or authenticated access.

### P0: ingestion controls

- The engine will fetch arbitrary remote `http(s)` audio and process it. See [streamingpath.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/routes/streamingpath.rs#L36).
- That is useful for a pilot, but risky in production.

What's missing:

- allowlists or presigned upload flow
- max file size and max duration enforcement
- content-type validation
- request timeout and egress safeguards
- customer storage isolation

### P1: customer-facing results storage and retention

- Results are cached and saved back through the storage abstraction, but there is no tenant-aware object ownership or retention policy. See [streamingpath.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/routes/streamingpath.rs#L76) and [backend.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/storage/backend.rs#L1).
- Before self-serve, you need object lifecycle and customer-level quotas.

### P1: packaging and commercial docs

- The engine has deploy docs and MCP docs, but not a commercial onboarding package: no auth docs, no pricing semantics, no SLAs, no production integration guide.
- That is fixable, but it matters for selling infra.

## 5. Marketing & Distribution

### 1. MCP-native distribution

- Channel: npm, GitHub, MCP ecosystem pages, Claude and MCP builder communities.
- Message: "Add audio transformation tools to your LLM in minutes."
- First action this week: tighten the MCP package and README around the hosted engine use case, not local hobby use.

### 2. Founder-led outbound to AI startups

- Channel: direct email and LinkedIn to teams building voice, transcription, or media agents.
- Message: "Stop building audio orchestration. Use a hosted engine with deterministic URLs, metadata, and MCP support."
- First action this week: outbound to 25 startups with one architecture diagram and one live processing demo.

### 3. Audio developer communities

- Channel: GitHub, KVR, audio software forums.
- Message: "Embed audio transforms in your app without shipping a full DSP backend."
- First action this week: post a technical launch showing the REST API, OpenAPI schema, and MCP tooling in one short repo demo.

## 6. Competitive Positioning

### Competitors and adjacent products

- Auphonic: API-integrated, workflow-oriented audio post-production with recurring and one-time credits plus white-label options.
- AudioShake: developer-oriented SDK and API around audio separation and embedded workflows.
- Descript: creator suite with text-based editing and AI audio/video workflows.
- Adobe Podcast: browser-based AI audio recording and editing with premium usage caps and workflow features.

### FreqModa streaming-engine wedge

- Do not position it against Descript or Adobe as a creator app.
- Position it as:
  - lightweight audio-processing infrastructure
  - deterministic URL-addressable transforms
  - deployable with S3, GCS, or filesystem backends
  - MCP-ready for agent workflows

That wedge is credible because the engine already combines REST endpoints, parameter preview, metadata extraction, storage abstraction, and MCP packaging in one stack. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L166), [streamingpath.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/routes/streamingpath.rs#L16), and [index.js](/Users/johannes/Repos/freqmoda/streaming-engine/mcp-server/index.js#L31).

## 7. Technical Debt vs. Revenue Trade-offs

- `/unsafe` is the largest commercial liability.
  - It is acceptable for demos, not for paid infra.
  - Mitigation: signed URLs or authenticated API gateway first.

- Arbitrary remote fetch is a growth hack and a security problem.
  - It accelerates pilots.
  - It will become a support, egress, and abuse problem at scale.
  - Mitigation: customer buckets, presigned URLs, allowlists.

- Current deployment defaults are too open.
  - Permissive CORS and unauthenticated Cloud Run are fine for testing, weak for paid service. See [startup.rs](/Users/johannes/Repos/freqmoda/streaming-engine/src/startup.rs#L198) and [deploy-streaming-engine.sh](/Users/johannes/Repos/freqmoda/scripts/deploy-streaming-engine.sh#L26).
  - Mitigation: private service, gateway, auth, and per-customer rate limits.

- No self-serve billing surface exists yet.
  - This is why the first sale should be a contract pilot, not checkout-led SaaS.

- Moat is not Rust alone.
  - Rust helps performance and operational efficiency, but the moat is only real if you combine:
    - reliability
    - low-latency processing
    - easy integration
    - MCP-native tooling
    - safe multi-tenant operations

## Recommended Path

1. Sell one hosted API and MCP pilot.
2. Add auth, keys, and usage metering.
3. Replace `/unsafe` in the paid path.
4. Convert to usage-based contract pricing with minimum commits.
5. Pursue embedded and white-label deals after the first references.
