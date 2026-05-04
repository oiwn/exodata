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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::functions::{
        CategoricalFieldSummary, ExoplanetDetail, HostIdentity,
        HostProvenanceSummary, HostStarSummary, HostSystemSummary,
        StellarHostDetail,
    };
    use std::collections::HashMap;

    #[test]
    fn website_schema_describes_site() {
        let schema = website_schema();

        assert_eq!(schema["@type"], "WebSite");
        assert_eq!(schema["name"], "Exodata");
        assert_eq!(schema["url"], "https://exodata.space/");
        assert_eq!(schema["inLanguage"], "en");
    }

    #[test]
    fn collection_schema_builds_absolute_url_and_parent_site() {
        let schema =
            collection_page_schema("Exoplanets", "Browse planets", "/exoplanets");

        assert_eq!(schema["@type"], "CollectionPage");
        assert_eq!(schema["url"], "https://exodata.space/exoplanets");
        assert_eq!(schema["isPartOf"]["name"], "Exodata");
    }

    #[test]
    fn stellarhost_dataset_schema_adds_spectral_keyword_and_encoded_url() {
        let host = StellarHostDetail {
            hostname: "Alpha Centauri A".to_string(),
            identity: HostIdentity {
                hostname: "Alpha Centauri A".to_string(),
                aliases: HashMap::new(),
            },
            system: HostSystemSummary::default(),
            star: HostStarSummary {
                spectype: Some(CategoricalFieldSummary {
                    key: "st_spectype".to_string(),
                    label: "Spectral Type".to_string(),
                    value: "G2 V".to_string(),
                    counts: vec![],
                    disputed: false,
                }),
                ..Default::default()
            },
            provenance: HostProvenanceSummary {
                record_count: 0,
                stellar_refs: vec![],
                system_refs: vec![],
                key_field_stats: vec![],
            },
            records: vec![],
            provenance_columns: vec![],
            metadata: HashMap::new(),
        };

        let schema = stellarhost_dataset_schema(&host);

        assert_eq!(schema["@type"], "Dataset");
        assert_eq!(
            schema["url"],
            "https://exodata.space/stellarhosts/Alpha%20Centauri%20A"
        );
        assert!(
            schema["keywords"]
                .as_array()
                .unwrap()
                .contains(&json!("G2 V"))
        );
    }

    #[test]
    fn exoplanet_dataset_schema_adds_host_and_method_keywords() {
        let detail = ExoplanetDetail {
            pl_name: "Kepler-10 b".to_string(),
            records: vec![json!({
                "hostname": "Kepler-10",
                "discoverymethod": "Transit"
            })],
            metadata: HashMap::new(),
        };

        let schema = exoplanet_dataset_schema(&detail);

        assert_eq!(schema["@type"], "Dataset");
        assert_eq!(
            schema["url"],
            "https://exodata.space/exoplanets/Kepler%2D10%20b"
        );
        assert!(
            schema["keywords"]
                .as_array()
                .unwrap()
                .contains(&json!("Kepler-10"))
        );
        assert!(
            schema["keywords"]
                .as_array()
                .unwrap()
                .contains(&json!("Transit"))
        );
    }
}
