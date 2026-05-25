pub struct DocPage {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
    pub markdown: &'static str,
}

pub static PAGES: &[DocPage] = &[
    DocPage {
        slug: "",
        title: "Exoplanets Catalog",
        description: "Learn how Exodata serves NASA Exoplanet Archive data through a server-rendered website, REST API, SQL endpoint, and local CLI.",
        keywords: &[
            "exoplanets",
            "NASA Exoplanet Archive",
            "stellar hosts",
            "REST API",
            "SQL",
        ],
        markdown: include_str!("../../../docs/about.md"),
    },
    DocPage {
        slug: "cli",
        title: "CLI Tools",
        description: "Use the Exodata CLI for local VOTable, Parquet, SQL, metadata, and curated insight workflows.",
        keywords: &["exodata CLI", "VOTable", "Parquet", "SQL", "insights"],
        markdown: include_str!("../../../docs/cli.md"),
    },
    DocPage {
        slug: "api",
        title: "REST API",
        description: "Use Exodata REST API endpoints for catalog tables, schema metadata, and read-only SQL queries.",
        keywords: &["Exodata API", "REST API", "OpenAPI", "SQL", "schema"],
        markdown: include_str!("../../../docs/api.md"),
    },
    DocPage {
        slug: "mcp",
        title: "MCP Server",
        description: "Connect coding agents to the Exodata read-only MCP server for catalog schema and SQL access. Includes Claude Code, Crush, OpenCode, and Codex CLI configuration.",
        keywords: &[
            "Exodata MCP",
            "Model Context Protocol",
            "agents",
            "Claude Code",
            "Codex",
        ],
        markdown: include_str!("../../../docs/mcp.md"),
    },
];

pub fn find_page(slug: &str) -> Option<&'static DocPage> {
    PAGES.iter().find(|page| page.slug == slug)
}

pub fn path_for(page: &DocPage) -> String {
    if page.slug.is_empty() {
        "/docs".to_string()
    } else {
        format!("/docs/{}", page.slug)
    }
}

pub fn route_for_doc_link(path: &str) -> Option<String> {
    let (file, anchor) = path.split_once('#').unwrap_or((path, ""));
    let base = match file {
        "about.md" => "/docs",
        "cli.md" => "/docs/cli",
        "api.md" => "/docs/api",
        "mcp.md" => "/docs/mcp",
        _ => return None,
    };

    if anchor.is_empty() {
        Some(base.to_string())
    } else {
        Some(format!("{base}#{anchor}"))
    }
}

#[cfg(test)]
mod tests {
    use super::route_for_doc_link;

    #[test]
    fn rewrites_known_docs_links() {
        assert_eq!(route_for_doc_link("about.md"), Some("/docs".to_string()));
        assert_eq!(
            route_for_doc_link("api.md#sql-query-endpoint"),
            Some("/docs/api#sql-query-endpoint".to_string())
        );
        assert_eq!(
            route_for_doc_link("mcp.md#connecting-an-agent"),
            Some("/docs/mcp#connecting-an-agent".to_string())
        );
        assert_eq!(route_for_doc_link("specs/cli.md"), None);
    }
}
