use leptos::prelude::*;

use crate::i18n::*;
use crate::locale::localized_path;

#[component]
pub fn HomepageManual() -> impl IntoView {
    let i18n = use_i18n();
    let docs_href = localized_path("/docs/mcp", i18n.get_locale_untracked());

    view! {
        <section id="mcp-exoplanet-data" class="homepage-manual" aria-labelledby="homepage-manual-title">
            <div class="homepage-manual__container">
                <span id="catalog-examples-title" class="block scroll-mt-24" aria-hidden="true"></span>
                <span id="mcp-setup-title" class="block scroll-mt-24" aria-hidden="true"></span>
                <h2 id="homepage-manual-title">{t!(i18n, manual.summary_title)}</h2>
                <p>{t!(i18n, manual.summary_description)}</p>
                <a class="homepage-manual__setup-link" href=docs_href>
                    {t!(i18n, manual.setup_link)}
                </a>
            </div>
        </section>
    }
}
