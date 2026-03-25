use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tracing::instrument;

use crate::state::AppStateDyn;

#[instrument(skip(state))]
pub async fn root_handler(
    State(state): State<AppStateDyn>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let keys = state.storage.list().await.map_err(|e| {
        tracing::error!("Failed to list audio files: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list files: {}", e),
        )
    })?;

    let mut rows = String::new();
    for key in &keys {
        rows.push_str(&format!(
            r#"<tr>
<td class="key"><a href="/unsafe/{key}">{key}</a></td>
<td class="actions">
<a href="/unsafe/{key}">stream</a>
| <a href="/meta/unsafe/{key}">meta</a>
| <a href="/params/unsafe/{key}">params</a>
</td>
<td class="player"><audio controls preload="none"><source src="/unsafe/{key}"></audio></td>
</tr>
"#
        ));
    }

    let count = keys.len();
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>freqmoda streaming engine</title>
<style>
body {{ font-family: verdana, arial, helvetica, sans-serif; font-size: 13px; color: #222; background: #f5f5f5; margin: 0; padding: 0; }}
#header {{ background: #1a1a2e; color: #ccc; padding: 8px 12px; font-size: 18px; font-weight: bold; }}
#header a {{ color: #7eb8da; text-decoration: none; }}
#header .subtitle {{ font-size: 11px; font-weight: normal; color: #888; }}
#nav {{ background: #16213e; padding: 4px 12px; font-size: 11px; }}
#nav a {{ color: #8ab4f8; text-decoration: none; margin-right: 12px; }}
#nav a:hover {{ text-decoration: underline; }}
#content {{ max-width: 960px; margin: 12px auto; background: #fff; border: 1px solid #ddd; padding: 0; }}
.info {{ padding: 8px 12px; background: #fafafa; border-bottom: 1px solid #eee; font-size: 11px; color: #666; }}
table {{ width: 100%; border-collapse: collapse; }}
th {{ text-align: left; background: #f0f0f0; padding: 6px 12px; font-size: 11px; color: #555; border-bottom: 2px solid #ddd; }}
td {{ padding: 6px 12px; border-bottom: 1px solid #eee; font-size: 12px; vertical-align: middle; }}
tr:hover {{ background: #f8f8ff; }}
td.key a {{ color: #0366d6; text-decoration: none; font-family: monospace; }}
td.key a:hover {{ text-decoration: underline; }}
td.actions {{ font-size: 11px; white-space: nowrap; }}
td.actions a {{ color: #555; text-decoration: none; }}
td.actions a:hover {{ color: #0366d6; text-decoration: underline; }}
td.player audio {{ height: 28px; vertical-align: middle; }}
.empty {{ padding: 24px; text-align: center; color: #999; }}
#footer {{ max-width: 960px; margin: 8px auto; font-size: 10px; color: #999; text-align: center; }}
</style>
</head>
<body>
<div id="header">
<a href="/">freqmoda</a> <span class="subtitle">streaming engine</span>
</div>
<div id="nav">
<a href="/">files</a>
<a href="/health">health</a>
<a href="/metrics">metrics</a>
<a href="/openapi.json">openapi</a>
<a href="/api-schema">api-schema</a>
</div>
<div id="content">
<div class="info">{count} file(s) in storage</div>
<table>
<tr><th>key</th><th>links</th><th>player</th></tr>
{rows}
</table>
</div>
<div id="footer">
freqmoda streaming engine · <a href="https://github.com/jonaylor89/freqmoda" style="color:#777">source</a>
</div>
</body>
</html>"#
    );

    Ok(Html(html))
}
