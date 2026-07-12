use crate::i18n::Locale;

pub const ZH_CN_PREFIX: &str = "/zh-CN";
pub const JA_PREFIX: &str = "/ja";

pub fn locale_from_path(path: &str) -> Locale {
    if has_prefix(path, ZH_CN_PREFIX) {
        Locale::zh_CN
    } else if has_prefix(path, JA_PREFIX) {
        Locale::ja
    } else {
        Locale::en
    }
}

pub fn strip_locale_prefix(path: &str) -> &str {
    for prefix in [ZH_CN_PREFIX, JA_PREFIX] {
        if path == prefix {
            return "/";
        }
        if let Some(rest) = path.strip_prefix(prefix)
            && rest.starts_with('/')
        {
            return rest;
        }
    }
    path
}

pub fn localized_path(path: &str, locale: Locale) -> String {
    let base = strip_locale_prefix(path);
    if is_utility_path(base) {
        return base.to_string();
    }

    match locale {
        Locale::en => base.to_string(),
        Locale::zh_CN => prefix_path(ZH_CN_PREFIX, base),
        Locale::ja => prefix_path(JA_PREFIX, base),
    }
}

pub fn localized_url(
    pathname: &str,
    search: &str,
    hash: &str,
    locale: Locale,
) -> String {
    format!("{}{}{}", localized_path(pathname, locale), search, hash)
}

pub fn is_utility_path(path: &str) -> bool {
    path == "/mcp"
        || path == "/swagger-ui"
        || path.starts_with("/rest/")
        || path.starts_with("/swagger-ui/")
        || path.starts_with("/sitemap-")
        || path.ends_with(".json")
        || path.ends_with(".csv")
}

fn has_prefix(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn prefix_path(prefix: &str, path: &str) -> String {
    if path == "/" || path.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_supported_locale_prefixes() {
        assert_eq!(locale_from_path("/"), Locale::en);
        assert_eq!(locale_from_path("/zh-CN"), Locale::zh_CN);
        assert_eq!(locale_from_path("/zh-CN/exoplanets"), Locale::zh_CN);
        assert_eq!(locale_from_path("/ja/stellarhosts"), Locale::ja);
        assert_eq!(locale_from_path("/zh-CN-not-a-locale"), Locale::en);
    }

    #[test]
    fn switches_locale_and_preserves_url_state() {
        assert_eq!(
            localized_url("/ja/exoplanets", "?page=2", "#results", Locale::zh_CN),
            "/zh-CN/exoplanets?page=2#results"
        );
        assert_eq!(
            localized_url("/zh-CN", "", "#mcp-exoplanet-data", Locale::en),
            "/#mcp-exoplanet-data"
        );
    }

    #[test]
    fn utility_and_export_paths_remain_unprefixed() {
        for path in [
            "/mcp",
            "/rest/query",
            "/swagger-ui",
            "/stellarhosts/TRAPPIST-1.json",
            "/exoplanets/Kepler-22%20b.csv",
            "/sitemap-index.xml",
        ] {
            assert_eq!(localized_path(path, Locale::ja), path);
        }
    }
}
