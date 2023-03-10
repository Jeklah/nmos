use std::sync::Arc;
use hyper::{Body, Method, Request, Response, StatusCode};
use serde::Serialize;
use futures::{FutureExt, TryFutureExt};
use crate::nmos::{make_sub_routes_body, make_route_pattern};
use crate::nmos::slog::Logger;
use crate::nmos::api_utils::{set_reply, make_filesystem_route, make_relative_path_content_type_handler};

pub fn make_api_sub_route(sub_route: &str, sub_router: hyper::service::make_service_fn, logger: Arc<Logger>) -> hyper::service::make_service_fn {
    let api = make_route_pattern("api", sub_route);

    let make_svc = move |req: Request<Body>| {
        let logger = logger.clone();

        let res = match (req.method(), req.uri().path()) {
            (&Method::GET, path) if path == "/" => {
                let body = make_sub_routes_body(&[sub_route], req);
                set_reply(Response::builder(), StatusCode::OK, Some(body))
            },
            (&Method::GET, path) if path == api.pattern => {
                let location = format!("{}{}", path, '/');
                Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header("Location", location)
                    .body(Body::empty())
                    .unwrap()
            },
            (&Method::GET, path) if path.starts_with(&(api.pattern + "/")) => {
                let sub_path = path.trim_start_matches(&(api.pattern + "/"));
                sub_router(req, sub_path.to_owned())
            },
            _ => {
                let body = format!("Method {} not allowed", req.method());
                set_reply(Response::builder(), StatusCode::METHOD_NOT_ALLOWED, Some(body))
            }
        };

        async move {
            logger.log_response(res.status().as_u16(), &req, &res);
            Ok::<_, hyper::Error>(res)
        }.boxed().unit_error().compat()
    };

    make_svc
}

pub fn make_admin_ui(filesystem_root: &str, logger: Arc<Logger>) -> hyper::service::make_service_fn {
    let valid_extensions = vec![
        ("ico", "image/x-icon"),
        ("html", "text/html"),
        ("js", "application/javascript"),
        ("map", "application/json"),
        ("json", "application/json"),
        ("css", "text/css"),
        ("png", "image/png")
    ];

    let content_type_handler = make_relative_path_content_type_handler(valid_extensions);
    let sub_router = make_filesystem_route(filesystem_root, content_type_handler, logger.clone());
    make_api_sub_route("admin", sub_router, logger)
}
