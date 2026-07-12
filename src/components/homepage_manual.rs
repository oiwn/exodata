use leptos::prelude::*;

use crate::components::docs::render::render_markdown;
use crate::i18n::*;
use crate::locale::localized_path;

const MANUAL_MARKDOWN_EN: &str = include_str!("../../docs/index.md");
const MANUAL_MARKDOWN_ZH_CN: &str =
    include_str!("../../docs/i18n/zh-CN/index.md");
const MANUAL_MARKDOWN_JA: &str = include_str!("../../docs/i18n/ja/index.md");
const HOST_PATH: &str = "/stellarhosts/TRAPPIST-1";
const PLANET_PATH: &str = "/exoplanets/Kepler-22%20b";
const MCP_ENDPOINT: &str = "https://exodata.space/mcp";

#[component]
pub fn HomepageManual() -> impl IntoView {
    let locale = use_i18n().get_locale_untracked();
    let markdown = match locale {
        Locale::en => MANUAL_MARKDOWN_EN,
        Locale::zh_CN => MANUAL_MARKDOWN_ZH_CN,
        Locale::ja => MANUAL_MARKDOWN_JA,
    };
    let html = render_markdown(markdown);

    view! {
        <section id="mcp-exoplanet-data" class="homepage-manual" aria-labelledby="homepage-manual-title">
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
    let i18n = use_i18n();
    let locale = i18n.get_locale_untracked();
    view! {
        <div class="homepage-manual__examples" aria-labelledby="catalog-examples-title">
            <div class="homepage-manual__section-heading">
                <h2 id="catalog-examples-title">
                    <a href="#catalog-examples-title">{t!(i18n, manual.examples_title)}</a>
                </h2>
                <p>{t!(i18n, manual.examples_subtitle)}</p>
            </div>

            <div class="homepage-manual__link-grid">
                <ExampleLink
                    href=localized_path(HOST_PATH, locale)
                    title=t_string!(i18n, manual.stellar_profile)
                    description=t_string!(i18n, manual.stellar_profile_description)
                />
                <ExampleLink
                    href=localized_path(PLANET_PATH, locale)
                    title=t_string!(i18n, manual.planet_profile)
                    description=t_string!(i18n, manual.planet_profile_description)
                />
                <ExampleLink
                    href="/stellarhosts/TRAPPIST-1.json".to_string()
                    title=t_string!(i18n, manual.host_json)
                    description=t_string!(i18n, manual.host_json_description)
                />
                <ExampleLink
                    href="/stellarhosts/TRAPPIST-1.csv".to_string()
                    title=t_string!(i18n, manual.host_csv)
                    description=t_string!(i18n, manual.host_csv_description)
                />
                <ExampleLink
                    href="/exoplanets/Kepler-22%20b.json".to_string()
                    title=t_string!(i18n, manual.planet_json)
                    description=t_string!(i18n, manual.planet_json_description)
                />
                <ExampleLink
                    href="/exoplanets/Kepler-22%20b.csv".to_string()
                    title=t_string!(i18n, manual.planet_csv)
                    description=t_string!(i18n, manual.planet_csv_description)
                />
                <ExampleLink
                    href=localized_path("/docs/api", locale)
                    title=t_string!(i18n, manual.rest_api)
                    description=t_string!(i18n, manual.rest_api_description)
                />
                <ExampleLink
                    href=localized_path("/docs/mcp", locale)
                    title=t_string!(i18n, manual.mcp_server)
                    description=t_string!(i18n, manual.mcp_server_description)
                />
                <ExampleLink
                    href=localized_path("/docs/cli", locale)
                    title=t_string!(i18n, manual.cli)
                    description=t_string!(i18n, manual.cli_description)
                />
                <ExampleLink
                    href="/swagger-ui".to_string()
                    title=t_string!(i18n, manual.swagger)
                    description=t_string!(i18n, manual.swagger_description)
                />
            </div>
        </div>
    }
}

#[component]
fn ExampleLink(
    href: String,
    title: &'static str,
    description: &'static str,
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
    let i18n = use_i18n();
    let docs_href = localized_path("/docs/mcp", i18n.get_locale_untracked());
    view! {
        <div class="homepage-manual__mcp" aria-labelledby="mcp-setup-title">
            <div class="homepage-manual__section-heading">
                <h2 id="mcp-setup-title">
                    <a href="#mcp-setup-title">{t!(i18n, manual.connect_title)}</a>
                </h2>
                <p>
                    {t!(i18n, manual.hosted_endpoint)} " "
                    <code>{MCP_ENDPOINT}</code>
                    ". " {t!(i18n, manual.read)} " "
                    <a
                        href="https://modelcontextprotocol.io/docs/getting-started/intro"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        {t!(i18n, manual.mcp_introduction)}
                    </a>
                    " " {t!(i18n, manual.or)} " "
                    <a href=docs_href>{t!(i18n, manual.mcp_docs)}</a>
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
                    label=t_string!(i18n, manual.codex_config)
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
                    label=t_string!(i18n, manual.opencode_config)
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
                    note=t_string!(i18n, manual.inspector_note)
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
    let i18n = use_i18n();
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
                    {move || if copied.get() {
                        t_string!(i18n, manual.copied)
                    } else {
                        t_string!(i18n, manual.copy)
                    }}
                </button>
            </div>
            <pre><code class=format!("language-{language}")>{code}</code></pre>
            {note.map(|note| view! { <p>{note}</p> })}
        </div>
    }
}

#[component]
fn AgentInteractionCard() -> impl IntoView {
    let i18n = use_i18n();
    let locale = i18n.get_locale_untracked();
    view! {
        <aside class="homepage-agent-card" aria-label=t_string!(i18n, manual.interaction_label)>
            <div class="homepage-agent-card__header">
                <span>"exodata"</span>
                <span>"CLI / MCP"</span>
            </div>
            <dl class="homepage-agent-card__flow">
                <div>
                    <dt>{t!(i18n, manual.ask)}</dt>
                    <dd>{t!(i18n, manual.ask_example)}</dd>
                </div>
                <div>
                    <dt>{t!(i18n, manual.tool)}</dt>
                    <dd>"describe_catalog -> query_catalog"</dd>
                </div>
                <div>
                    <dt>{t!(i18n, manual.sql)}</dt>
                    <dd>
                        <code>"SELECT hostname, sy_dist, sy_pnum FROM stellarhosts WHERE sy_dist IS NOT NULL ORDER BY sy_dist LIMIT 5"</code>
                    </dd>
                </div>
                <div>
                    <dt>{t!(i18n, manual.result)}</dt>
                    <dd>{t!(i18n, manual.result_example)}</dd>
                </div>
            </dl>
            <div class="homepage-agent-card__links">
                <a href=localized_path("/docs/cli", locale)>{t!(i18n, manual.cli_docs)}</a>
                <a href=localized_path("/docs/mcp", locale)>{t!(i18n, manual.short_mcp_docs)}</a>
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
