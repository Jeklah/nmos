use crate::nmos::{api_utils::*, filesystem_route::*, slog::*, *};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::{header::HeaderValue, header::LOCATION, Response, StatusCode};
use warp::{Filter, Reply};

pub fn make_api_sub_route(
    sub_route: &str,
    sub_router: warp::filters::BoxedFilter<(impl Reply,)>,
    gate: Arc<Mutex<Box<dyn Gate>>>,
) -> warp::filters::BoxedFilter<(impl Reply,)> {
    let api = format!("/api/{}", sub_route);
    let sub_routes = format!("{}/*", sub_route);

    let router = warp::any().and_then(move || {
        let gate = gate.clone();
        let api_clone = api.clone();

        async move {
            Ok(Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .header("Access-Control-Max-Age", "86400")
                .header("Cache-Control", "no-cache, no-store, must-revalidate")
                .header("Pragma", "no-cache")
                .header("Expires", "0")
                .header("X-Content-Type-Options", "nosniff")
                .header("X-Frame-Options", "SAMEORIGIN")
                .header("X-XSS-Protection", "1; mode=block")
                .header("Content-Security-Policy", "default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self'; connect-src 'self'")
                .header("Referrer-Policy", "strict-origin-when-cross-origin")
                .status(StatusCode::OK)
                .body(make_sub_routes_body(
                    &[sub_routes.as_str().to_owned()],
                    &api_clone,
                    &HeaderValue::from_str("http://localhost:3209/").unwrap(),
                    None,
                    gate.lock().await.deref_mut(),
                ))?)
        }
    });

    warp::path::end()
        .and(warp::get())
        .and(router)
        .boxed()
        .or(warp::path(api.as_str())
            .and(warp::get())
            .map(|| warp::redirect::temporary(Uri::try_from(sub_routes).unwrap())))
        .or(warp::path(api)
            .and(warp::path::tail())
            .and(sub_router)
            .boxed())
        .boxed()
}

pub fn make_admin_ui(
    filesystem_root: &str,
    gate: Arc<Mutex<Box<dyn Gate>>>,
) -> warp::filters::BoxedFilter<(impl Reply,)> {
    // To serve the admin UI, only a few HTML, JavaScript and CSS files are necessary
    let valid_extensions: HashMap<&str, &str> = vec![
        ("ico", "image/x-icon"),
        ("html", "text/html"),
        ("js", "application/javascript"),
        ("map", "application/json"),
        ("json", "application/json"),
        ("css", "text/css"),
        ("png", "image/png"),
    ]
    .into_iter()
    .collect();

    let handler = make_relative_path_content_type_handler(valid_extensions);

    make_api_sub_route(
        "admin",
        make_filesystem_route(filesystem_root, handler, gate),
        gate,
    )
}
