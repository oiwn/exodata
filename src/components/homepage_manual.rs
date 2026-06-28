use leptos::prelude::*;

use crate::components::docs::render::render_markdown;

const MANUAL_MARKDOWN: &str = include_str!("../../docs/index.md");
const HOST_NAME: &str = "TRAPPIST-1";
const HOST_PATH: &str = "/stellarhosts/TRAPPIST-1";
const PLANET_NAME: &str = "Kepler-22 b";
const PLANET_PATH: &str = "/exoplanets/Kepler-22%20b";
const MCP_ENDPOINT: &str = "https://exodata.space/mcp";

#[component]
pub fn HomepageManual() -> impl IntoView {
    let html = render_markdown(MANUAL_MARKDOWN);

    view! {
        <section id="homepage-manual" class="homepage-manual" aria-labelledby="homepage-manual-title">
            <div class="homepage-manual__container">
                <div class="homepage-manual__intro">
                    <article
                        id="homepage-manual-title"
                        class="homepage-manual__prose"
                        inner_html=html
                    ></article>
                    <AgentInteractionCard/>
                </div>

                <ExampleLinks/>
                <McpSetup/>
            </div>
        </section>
    }
}

#[component]
fn ExampleLinks() -> impl IntoView {
    view! {
        <div class="homepage-manual__examples" aria-labelledby="catalog-examples-title">
            <div class="homepage-manual__section-heading">
                <h2 id="catalog-examples-title">"Open a real catalog record"</h2>
                <p>"Stable routes make pages, exports, API docs, and tool setup easy to share."</p>
            </div>

            <div class="homepage-manual__link-grid">
                <ExampleLink
                    href=HOST_PATH
                    title="Stellar host profile"
                    description=format!("Open the {HOST_NAME} host-system detail page.")
                />
                <ExampleLink
                    href=PLANET_PATH
                    title="Planet profile"
                    description=format!("Open the {PLANET_NAME} exoplanet detail page.")
                />
                <ExampleLink
                    href="/stellarhosts/TRAPPIST-1.json"
                    title="Host JSON"
                    description="Download the full host detail payload.".to_string()
                />
                <ExampleLink
                    href="/stellarhosts/TRAPPIST-1.csv"
                    title="Host CSV"
                    description="Download matching host source rows.".to_string()
                />
                <ExampleLink
                    href="/exoplanets/Kepler-22%20b.json"
                    title="Planet JSON"
                    description="Download the full planet detail payload.".to_string()
                />
                <ExampleLink
                    href="/exoplanets/Kepler-22%20b.csv"
                    title="Planet CSV"
                    description="Download matching planet source rows.".to_string()
                />
                <ExampleLink
                    href="/docs/api"
                    title="REST API"
                    description="Query tables, schemas, details, insights, and SQL endpoints.".to_string()
                />
                <ExampleLink
                    href="/docs/mcp"
                    title="MCP server"
                    description="Connect an AI client to inspect schema and query the catalog.".to_string()
                />
                <ExampleLink
                    href="/docs/cli"
                    title="CLI"
                    description="Query live or local data from the terminal.".to_string()
                />
                <ExampleLink
                    href="/swagger-ui"
                    title="Swagger UI"
                    description="Explore the OpenAPI schema interactively.".to_string()
                />
            </div>
        </div>
    }
}

#[component]
fn ExampleLink(
    href: &'static str,
    title: &'static str,
    description: String,
) -> impl IntoView {
    view! {
        <a class="homepage-manual-link" href=href>
            <span class="homepage-manual-link__title">{title}</span>
            <span class="homepage-manual-link__description">{description}</span>
        </a>
    }
}

#[component]
fn McpSetup() -> impl IntoView {
    view! {
        <div class="homepage-manual__mcp" aria-labelledby="mcp-setup-title">
            <div class="homepage-manual__section-heading">
                <h2 id="mcp-setup-title">"Connect an MCP client"</h2>
                <p>
                    "The hosted endpoint is "
                    <code>{MCP_ENDPOINT}</code>
                    ". Read the "
                    <a
                        href="https://modelcontextprotocol.io/docs/getting-started/intro"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "MCP introduction"
                    </a>
                    " or the "
                    <a href="/docs/mcp">"Exodata MCP docs"</a>
                    "."
                </p>
            </div>

            <div class="homepage-manual__command-grid">
                <CommandBox
                    label="Codex CLI"
                    language="bash"
                    code="codex mcp add exodata \
    --url https://exodata.space/mcp"
                />
                <CommandBox
                    label="Codex config fallback"
                    language="toml"
                    code="[mcp_servers.exodata]
    url = \"https://exodata.space/mcp\""
                />
                <CommandBox
                    label="Claude Code"
                    language="bash"
                    code="claude mcp add \
    --transport http \
    exodata \
    https://exodata.space/mcp"
                />
                <CommandBox
                    label="OpenCode CLI"
                    language="bash"
                    code="opencode mcp add exodata \
    --url https://exodata.space/mcp"
                />
                <CommandBox
                    label="OpenCode config"
                    language="json"
                    code="{
  \"$schema\": \"https://opencode.ai/config.json\",
  \"mcp\": {
    \"exodata\": {
      \"type\": \"remote\",
      \"url\": \"https://exodata.space/mcp\",
      \"enabled\": true
    }
  }
}"
                />
                <CommandBox
                    label="MCP Inspector"
                    language="bash"
                    code="npx @modelcontextprotocol/inspector"
                    note="Open the Inspector UI, then connect to the hosted endpoint with Streamable HTTP transport."
                />
            </div>
        </div>
    }
}

#[component]
fn CommandBox(
    label: &'static str,
    language: &'static str,
    code: &'static str,
    #[prop(optional)] note: Option<&'static str>,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);

    view! {
        <div class="homepage-command-box">
            <div class="homepage-command-box__header">
                <span>{label}</span>
                <button
                    type="button"
                    class="homepage-command-box__copy"
                    on:click=move |_| copy_code_to_clipboard(code, set_copied)
                >
                    {move || if copied.get() { "Copied" } else { "Copy" }}
                </button>
            </div>
            <pre><code class=format!("language-{language}")>{code}</code></pre>
            {note.map(|note| view! { <p>{note}</p> })}
        </div>
    }
}

#[component]
fn AgentInteractionCard() -> impl IntoView {
    view! {
        <aside class="homepage-agent-card" aria-label="Example catalog tool interaction">
            <div class="homepage-agent-card__header">
                <span>"exodata"</span>
                <span>"CLI / MCP"</span>
            </div>
            <dl class="homepage-agent-card__flow">
                <div>
                    <dt>"Ask"</dt>
                    <dd>"nearest known planet-hosting systems with distance and planet count"</dd>
                </div>
                <div>
                    <dt>"Tool"</dt>
                    <dd>"describe_catalog -> query_catalog"</dd>
                </div>
                <div>
                    <dt>"SQL"</dt>
                    <dd>
                        <code>"SELECT hostname, sy_dist, sy_pnum FROM stellarhosts WHERE sy_dist IS NOT NULL ORDER BY sy_dist LIMIT 5"</code>
                    </dd>
                </div>
                <div>
                    <dt>"Result"</dt>
                    <dd>"structured rows ready for a table, notebook, or follow-up query"</dd>
                </div>
            </dl>
            <div class="homepage-agent-card__links">
                <a href="/docs/cli">"CLI docs"</a>
                <a href="/docs/mcp">"MCP docs"</a>
            </div>
        </aside>
    }
}

fn copy_code_to_clipboard(text: &str, set_copied: WriteSignal<bool>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;

        if let Some(window) = web_sys::window() {
            copy_text(text);
            set_copied.set(true);

            let reset = wasm_bindgen::closure::Closure::once(move || {
                set_copied.set(false);
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                reset.as_ref().unchecked_ref(),
                1800,
            );
            reset.forget();
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (text, set_copied);
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function copyText(text) {
  function fallbackCopy(value) {
    const textarea = document.createElement('textarea');
    textarea.value = value;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.left = '-9999px';
    textarea.style.top = '0';
    document.body.appendChild(textarea);
    textarea.select();
    try {
      document.execCommand('copy');
    } finally {
      document.body.removeChild(textarea);
    }
  }

  if (navigator.clipboard && window.isSecureContext) {
    navigator.clipboard.writeText(text).catch(() => fallbackCopy(text));
    return;
  }

  fallbackCopy(text);
}"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = copyText)]
    fn copy_text(text: &str);
}
