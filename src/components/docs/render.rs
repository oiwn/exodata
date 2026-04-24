use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html::push_html};

use super::registry;

pub fn render_markdown(markdown: &str) -> String {
    let parser =
        Parser::new_ext(markdown, markdown_options()).map(|event| match event {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let dest_url = registry::route_for_doc_link(dest_url.as_ref())
                    .map(CowStr::from)
                    .unwrap_or(dest_url);

                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                })
            }
            _ => event,
        });

    let mut html = String::new();
    push_html(&mut html, parser);
    html
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
}

#[cfg(test)]
mod tests {
    use super::render_markdown;

    #[test]
    fn renders_markdown_and_rewrites_docs_links() {
        let html = render_markdown("[API](api.md#sql-query-endpoint)");
        assert!(html.contains("href=\"/docs/api#sql-query-endpoint\""));
    }
}
