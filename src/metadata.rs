use crate::server::functions::ColumnMetadata;
use leptos::prelude::*;
use std::collections::HashMap;

pub const METADATA_SCRIPT_ID: &str = "__EXO_METADATA__";

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppMetadata {
    pub stellarhosts: HashMap<String, ColumnMetadata>,
    pub exoplanets: HashMap<String, ColumnMetadata>,
}

pub type AppMetadataStore = RwSignal<AppMetadata>;

/// Provides a single global metadata store for all table pages.
pub fn provide_app_metadata_store() {
    if use_context::<AppMetadataStore>().is_some() {
        return;
    }

    if let Some(metadata) = use_context::<AppMetadata>() {
        provide_context(RwSignal::new(metadata));
        return;
    }

    #[cfg(feature = "hydrate")]
    if let Some(metadata) = read_embedded_metadata() {
        provide_context(RwSignal::new(metadata));
        return;
    }

    provide_context(RwSignal::new(AppMetadata::default()));
}

pub fn use_app_metadata_store() -> AppMetadataStore {
    expect_context::<AppMetadataStore>()
}

#[cfg(feature = "hydrate")]
fn read_embedded_metadata() -> Option<AppMetadata> {
    let window = leptos::web_sys::window()?;
    let document = window.document()?;
    let payload = document
        .get_element_by_id(METADATA_SCRIPT_ID)?
        .text_content()?;

    leptos::serde_json::from_str::<AppMetadata>(&payload).ok()
}
