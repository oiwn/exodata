use leptos::prelude::*;
use leptos::serde_json::{Value, json};

use crate::metadata_helpers::{
    SITE_NAME, SITE_URL, exoplanet_detail_description, overview_description,
    stellarhost_detail_description,
};
use crate::server::functions::{ExoplanetDetail, StellarHostDetail};

#[component]
pub fn StructuredData(value: Value) -> impl IntoView {
    let payload = value.to_string().replace("</", "<\\/");

    view! {
        <script type="application/ld+json">
            {payload}
        </script>
    }
}

pub fn website_schema() -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "WebSite",
        "name": SITE_NAME,
        "url": format!("{SITE_URL}/"),
        "description": overview_description(),
        "inLanguage": "en"
    })
}

pub fn collection_page_schema(
    name: &str,
    description: &str,
    path: &str,
) -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        "name": name,
        "url": absolute_url(path),
        "description": description,
        "isPartOf": {
            "@type": "WebSite",
            "name": SITE_NAME,
            "url": format!("{SITE_URL}/")
        }
    })
}

pub fn stellarhost_dataset_schema(host: &StellarHostDetail) -> Value {
    let mut keywords = vec!["stellar host".to_string(), "host star".to_string()];

    if let Some(spectype) = host.star.spectype.as_ref() {
        keywords.push(spectype.value.clone());
    }

    json!({
        "@context": "https://schema.org",
        "@type": "Dataset",
        "name": format!("{} stellar host dataset", host.hostname),
        "url": absolute_url(&format!("/stellarhosts/{}", encode_segment(&host.hostname))),
        "description": stellarhost_detail_description(host),
        "isAccessibleForFree": true,
        "keywords": keywords,
        "includedInDataCatalog": {
            "@type": "DataCatalog",
            "name": SITE_NAME,
            "url": format!("{SITE_URL}/")
        }
    })
}

pub fn exoplanet_dataset_schema(detail: &ExoplanetDetail) -> Value {
    let mut keywords = vec!["exoplanet".to_string()];

    if let Some(record) = detail.records.first() {
        if let Some(hostname) = record.get("hostname").and_then(|v| v.as_str()) {
            keywords.push(hostname.to_string());
        }
        if let Some(method) =
            record.get("discoverymethod").and_then(|v| v.as_str())
        {
            keywords.push(method.to_string());
        }
    }

    json!({
        "@context": "https://schema.org",
        "@type": "Dataset",
        "name": format!("{} exoplanet dataset", detail.pl_name),
        "url": absolute_url(&format!("/exoplanets/{}", encode_segment(&detail.pl_name))),
        "description": exoplanet_detail_description(detail),
        "isAccessibleForFree": true,
        "keywords": keywords,
        "includedInDataCatalog": {
            "@type": "DataCatalog",
            "name": SITE_NAME,
            "url": format!("{SITE_URL}/")
        }
    })
}

fn absolute_url(path: &str) -> String {
    if path.is_empty() || path == "/" {
        format!("{SITE_URL}/")
    } else {
        format!("{SITE_URL}{path}")
    }
}

fn encode_segment(value: &str) -> String {
    percent_encoding::utf8_percent_encode(
        value,
        percent_encoding::NON_ALPHANUMERIC,
    )
    .to_string()
}
