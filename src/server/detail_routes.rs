use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::locale::strip_locale_prefix;

/// Normalize website detail paths before the Leptos route matcher sees them.
pub async fn normalize_detail_route(
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return next.run(request).await;
    }

    let path = request.uri().path();
    let base = strip_locale_prefix(path);
    let Some(segment) = base
        .strip_prefix("/stellarhosts/")
        .or_else(|| base.strip_prefix("/exoplanets/"))
    else {
        return next.run(request).await;
    };
    let segment = segment.trim_end_matches('/');
    if segment.ends_with(".json") || segment.ends_with(".csv") {
        return next.run(request).await;
    }
    if segment.is_empty() || segment.contains('/') {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !valid_segment(segment) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if path.ends_with('/') {
        let mut location = path.trim_end_matches('/').to_string();
        if let Some(query) = request.uri().query() {
            location.push('?');
            location.push_str(query);
        }
        return (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, location)],
        )
            .into_response();
    }
    next.run(request).await
}

fn valid_segment(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return false;
            }
            index += 3;
        } else {
            index += 1;
        }
    }
    percent_encoding::percent_decode_str(segment)
        .decode_utf8()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, middleware};
    use tower::ServiceExt;

    async fn response(method: Method, path: &str) -> Response {
        Router::new()
            .fallback(|| async { StatusCode::OK })
            .layer(middleware::from_fn(normalize_detail_route))
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn seo_detail_routes_redirect_slashes_and_preserve_query_and_locale() {
        for prefix in ["", "/zh-CN", "/ja"] {
            for entity in ["stellarhosts", "exoplanets"] {
                let path =
                    format!("{prefix}/{entity}/Kepler%2D22%20b///?filter=a%2Bb");
                for method in [Method::GET, Method::HEAD] {
                    let result = response(method, &path).await;
                    assert_eq!(result.status(), StatusCode::MOVED_PERMANENTLY);
                    assert_eq!(
                        result.headers()[header::LOCATION],
                        format!("{prefix}/{entity}/Kepler%2D22%20b?filter=a%2Bb")
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn seo_detail_routes_validate_names_without_changing_lookup_semantics()
    {
        for path in ["/exoplanets/%", "/exoplanets/%ZZ", "/ja/stellarhosts/%FF"] {
            assert_eq!(
                response(Method::GET, path).await.status(),
                StatusCode::BAD_REQUEST
            );
        }
        for path in ["/exoplanets/", "/stellarhosts/name/extra"] {
            assert_eq!(
                response(Method::GET, path).await.status(),
                StatusCode::NOT_FOUND
            );
        }
        for path in [
            "/exoplanets/Kepler-22+b",
            "/exoplanets/%3Cscript%3E",
            "/exoplanets/name%2Fpart",
            "/rest/query/",
            "/mcp/",
            "/pkg/file/",
            "/exoplanets/name.csv/",
            "/ja/exoplanets/name.json/",
        ] {
            assert_eq!(
                response(Method::GET, path).await.status(),
                StatusCode::OK
            );
        }
        assert_eq!(
            response(Method::POST, "/exoplanets/%ZZ/").await.status(),
            StatusCode::OK
        );
    }
}
