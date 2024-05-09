use serde;
use http::{Request, Response, StatusCode, header::{HeaderValue, HeaderMap}};
use serde_json::{json, Value};
use regex::Regex;
use url;

use serde::Serialize;
use std::collections:{BTreeMap, HashSet};
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
        for(), v) in obj.iter_mut() {
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
