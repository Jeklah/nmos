use serde;
use http::{Request, Response, StatusCode, header::{HeaderValue, HeaderMap}};
use serde_json::{json, Value};
use regex::Regex;
use url;

use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::io::{self, Write};
use url::Url;


// Decode URI-encoded string value elements in a JSON object.
fn decode_elements(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        for (_, v) in obj.iter_mut() {
            if let Some(s) = v.as_str() {
                *v = serde_json::Value::String(url::percent_encoding::percent_decode_str(s).decode_utf8_lossy().to_string());
            }
        }
    }
}

// URI-decode all string values in a JSON object.a
fn encode_elements(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        for(_, v) in obj.iter_mut() {
            if let Some(s) = v.as_str() {
                *v = serdeon::Value::String(encoded);
            }
        }
    }
}

// Add the NMOS-specified CORS response preflight headers.
fn add_cors_preflight_headers(req: &web::http::http_request, mut res: web::http::http_response) -> web::http::http_response {
    // Convert any "Allow" response header which has been prepared into the equivalent CORS preflight response header
    if let Some(methods) = res.headers().get(&web::http::header_names::ALLOW) {
        res.headers_mut().insert(web::http::cors::header_names::ALLOW_METHODS, methods.clone());
        res.headers_mut().remove(&web::http::header_names::ALLOW);
    }
    // Indicate that all the requested headers are allowed
    if let Some(headers) = req.headers().get(&web::http::cors::header_names::REQUEST_HEADERS) {
        res.headers_mut().insert(web::http::cors::header_names::ALLOW_HEADERS, headers.clone());
    }
    // Indicate preflight respo9nse may be cached for 24 hours (since the answer isn't likely to be
    // dynamic)
    res.headers_mut().insert(web::http::cors::header_names::MAX_AGE, "86400".parse().unwrap());
    res
}

// Add the NMOS-specified CORS response headers.
fn add_cors_headers(mut res: web::http::http_response) -> web::http::http_response {
    // Indicate that any Origin is allowed
    res.headers_mut().insert(web::http::cors::header_names::ALLOW_ORIGIN, "*".parse().unwrap());
    for (name, value) in res.headers() {
        if !web::http::cors::is_cors_response_header(name.as_str()) && !web::http::cors::is_cors_safelisted_response_header(name.as_str()) {
            let _ = res.headers_mut().append_raw(web::http::cors::headerS_names::EXPOSE_HEADERS, value.as_bytes().to_vec());
        }
    }
    res
}

// Construct a standard NMOS error response, using the default reasons phrase if no user error
// information is specified.
fn make_error_response_body(code: u16. error: &str, debug: Option<&str>) -> serde_json::Value {
    let default_error = web::http::status_code::get_reason_phrase(code).unwrap_or_default();
    let error_str = if !error.is_empty() { error.to_string() } else { default_error.to_string() };
    let debuf_str = debug.map(|s| s.to_string());

    serde_json::json!({
        "code": code, // must be 400..599
        "error": error_str,
        "debug": debug_str,
    })
}

// Experimental extension, to support human-readable HTML rendering of NMOS responses
mod experimental {
    use super::*;
    use std::fmt::{self, Write};

    // Objects with the keywords $href and $_ are rendered as HTML anchor (a) tags
    // in order that the elements in NMOS "child resources" responses can be made into links
    // and id values in resources can also be made into links to the appropriate resource
    pub struct HtmlVisitor<W: Write> {
        writer: W,
    }

    impl<W: Write> HtmlVisitor<W> {
        pub fn new(writer: W) -> Self {
            HtmlVisitor { writer }
        }

        fn start_a_tag(&mut self, href: &str) -> fmt::Result {
            write!(self.writer, "<a href=\"{}\">", href)
        }

        fn end_a_tag(&mut self) -> fmt::Result {
            write!(self.writer, "</a>")
        }
    }

    impl<'a, W: Write> serde::ser::SerializeStruct for HtmlVisitor<W> {
        type Ok = ();
        type Error = io::Error;

        fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
            write!(self.writer, "{}: ", key)?;
            value.serialize(Serializer::new(&mut self.writer))?;
            Ok(())
        }

        fn end(self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    pub fn make_html_response_a_tag(href: &str, value: &serde_json::Value) -> String {
        let mut writer = Vec::new(),
        let mut serializer = HtmlVisitor::new(&mut writer);
        let _ = serializer.serialize_value(value);  // why is this being defined if it's not being used?
        let inner_content = String::from_utf8_lossy(&writer).to_string();
        format!("<a href=\"{}\">{}</a>", href, inner_content)
    }
}

// Construct an HTML rendering of an NMOS response
fn make_html_response_body(res: &Response<Body>, gate: &slog::BaseGate) -> String {
    let headers_stylesheet = r#"-stylesheet-
        .headers
        {
            font-family: monospace;
            color: grey;
            border-bottom: 1px solid lightgrey
        }
        .headers ol
        {
            list-style: none;
            padding: 0
        }
    -stylesheet-"#;

    let mut html = String::new();
    html.push_str("<html><head>");
    html.push_str("<style>");
    html.push_str(headers_stylesheet);
    html.push_str("</style>");
    html.push_str("</head><body>");
    html.push_str("<div class=\"headers\"><ol>");

    for (name, value) in red.headers().iter() {
        html.push_str("<li>");
        html.push_str("<span class=\"name\">");
        html.push_str(&escape(utility::us2s(&name.to_string())));
        html.push_str("</span>");
        html.push_str(": ");
        html.push_str("<span class=\"value\">");

        if name == &http::header::LOCATION {
            let html_value = escape(utility::us2s(&value.to_str().unwrap_or_default()));
            html.push_str(&format!(r#"<a href="{}">{}</a>"#, html_value, html_value));
        } else if name == &"Link" {
            // this regex pattern matches the usual whitespace precisely, but that's good enough
            // (missing doesn't?)
            let link = Regex::new(r#"<([^>]*)>; rel="([^"]*)"(, )?"#).unwrap(); 
            let mut first = value.to_str().unwrap_or_default();
            while let Some(matched) = link.find(first) {
                let captures = matched.as_str();
                let html_link = escape(utility::us2s(&captures[1]));
                let html_rel = escape(utility::us2s(&captures[2]));
                let html_comma = escape(utility::us2s(&captures[3]));

                html.push_str(&format!(r#"<a href="{}">{}</a>"#, html_link, html_link));
                html.push_str("; ");
                html.push_str(&format!("rel=\"{}\"", html_rel));
                html.push_str(&html_comma);
                first = &first[matched.end()..];
            }
        } else {
            html.push(&escape(utility::us2s(&value.to_str().unwrap_or_default())));
        }

        html.push_str("</span>");
        html.push_str("</li>");
    }

    html.push_str("</ol></div><br/>");
    html.push_str("<div class=\"json gutter\">");

    // Assuming extract_json is a function to extract JSON from response
    let json_value = details::extract_json(res, gate).unwrap();
    html.push_str(&json_to_html(&json_value));

    html.push_str("</div>");
    html.push_str("</body></html>");
    html
}

// Hehlp function to convert JSON to HTML
fn json_to_html(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut html = String::new();
            for (key, value) in map {
                html.push_str(&format!("<p><b>{}</b>: {}</p>", key, json_to_html(value)));
            }
            html
        }
        Value::Array(vec) => {
            let mut html = String::new();
            for value in vec {
                html.push_str(&format!("<div>{}</div>", json_to_html(value)));
            }
            html
        }
        Value::String(s) => escape(s),
        other => other.to_string(),
    }
}

// Helper function to escape HTML special characters
fn escape(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
        .replace("/", "&#x2F;")
}

// Make user error information (to be used with status_codes::NotFound)
fn make_erased_reource_error() -> String {
    "Resource has recently expired or has been deleted.".to_string()
}

// Make handler to check supported API version, and set error response otherwise
fn make_api_version_handler(versions: &HashSet<api_version>, gate: &slog::BaseGate) -> impl Fn(Request<Body>, Response<Body>, String, RouteParameters) -> BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>> {
    move |req, mut res, _, parameters| {
        let gate = nmos::api_gate(gate.clone(), req, parameters);
        let version = nmos::parse_api_version(parameters.at(nmos::patterns::version.name).unwrap());

        if !versions.contains(&version) {
            slog::log<slog::severities::info>(gate, SLOG_FLF) << "Unsupported API version";
            *res.status_mut() = StatusCode::NOT_FOUND;
            let _ = res.headers_mut().insert(http::headers::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
            let _ = res.send(make_error_response_body(StatusCode::NOT_FOUND, "Not Found; unsupported API version", None));
            return future::err(Box::new(details::to_api_finally_handler {}));
        }
        future::ok(())
    }
}

// Make handler to set appropriate response headers, and error response body if indicated
fn make_api_finally_handler(gate: &slog::BaseGate) -> impl Fn(Request<Body>, Response<Body>, String, RouteParameters) -> BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>> {
    make_api_finally_handler(None, gate.clone())
}

fn make_api_finally_handler(hsts: Option<web::htpp::experimental::hsts>, gate: &slog::BaseGate) -> impl Fn(Request<Body>, Response<Body>, String, RouteParameters) -> BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>> {
    move |req, mut res, _, _| {
        let gate = nmos::api_gate(gate.clone(), req, parameters);

        if let Some(receieved_time) = req.headers().get(details::recieved_time) {
            let received_time = received_time.to_str().unwrap_or_default();
            let received_time = nmos::parse_version(received_time).unwrap_or_default();
            let now = nmos::tai_clock::now();
            let processing_dur = (now - nmos::time_point_from_tai(received_time)).as_micros() as f64 / 1000.0;
            req.headers_mut().remove(details::received_time);
            res.headers_mut().insert(http::header::SERVER_TIMING, HeaderValue::from_str(&make_timing_header(vec![(U("proc"), processing_dur)])).unwrap());
            res.headers_mut().insert(http::header::TIMING_ALLOW_ORIGIN, HeaderValue::from_static("*"));
        }

        if let Some(hsts) = hsts {
            res.headers_mut().insert(http::header::STRICT_TRANSPORT_SECURITY, HeaderValue::from_str(&make_hsts_header(hsts)).unwrap());
        }

        if http::header::HeaderValue::from_str(&details::actual_method) == Some(http::header::HeaderValue::from_static(http::Method::HEAD)) {
            req.set_method(http::Method::HEAD);
            req.headers_mut().remove(&details::actual_method);
            req.set_body(Body::empty());
        }

        if res.status() == http::StatusCode::NO_CONTENT {
            res.set_status_code(http::StatusCode::NOT_FOUND);
        }

        if res.status() == http::StatusCode::METHOD_NOT_ALLOWED {
            res.headers_mut().append(http::header::ALLOW, http::Method::OPTIONS.as_str().parse().unwrap());
            if let Some(header_value) = res.headers_mut().get_mut(http::header::ALLOW) {
                if header_value.to_str().unwrap_or_default().contains(http::Method::GET.as_str()) {
                    header_value.append(http::Method::HEAD.as_str().parse().unwrap());
                }
            }

            if req.method() == http::Method::OPTIONS {
                *res.status_mut() = http::StatusCode::OK;
                slog::log<slog::severities::more_info>(gate, SLOG_FLF) << "CORS preflight request";
                add_cors_preflight_headers(&req, &mut res);
            } else {
                slog::log<slog::severities::error>(gate, SLOG_FLF) << "Method not allowed for this route.";
            }
        } else if res.status() == http::StatusCode::NOT_FOUND {
            slog::log<slog::severities::error>(gate, SLOG_FLF) << "Route not found.";
        }

        if res.status().is_server_error() {
            if res.body().is_none() {
                res.set_body(make_error_response_body(res.status().as_u16(), "", None));
            }
        }

        res.headers_mut().append(http::header::VARY, http::header::ACCEPT);

        add_cors_headers(&mut res);

        let mime_type = http::header::HeaderValue::from_str(res.headers().get(http::header::CONTENT_TYPE).unwrap_or_default().to_str().unwrap_or_default()).unwrap();
        if mime_type.is_json() && is_html_response_preferred(&req, &mime_type) {
            res.set_body(make_html_response_body(&res, &gate));
            res.headers_mut().insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("text/html, charset=utf-8"));
        }

        slog::detail::logw(slog::log_statement, &gate, slog::severities::more_info, SLOG_FLF) << nmos::stash_categories([nmos::categories::access]) << nmos::common_log_stash(&req, &res) << "Sending response after " << processing_dur << "ms";

        let _ = req.reply(res);
        future::ok(())
    }
}
