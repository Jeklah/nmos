use http::{header, StatusCode};
use std::collections::{HashMap, HashSet};
use url::form_urlencoded;

// Decode URI-encoded string value elements in a JSON object
fn decode_elements(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        for (_, v) in obj.iter_mut() {
            if let Some(s) = v.as_str() {
                *v = serde_json::Value::String(
                    url::percent_encoding::percent_decode_str(s)
                        .decode_utf8_lossy()
                        .into_owned(),
                );
            }
        }
    }
}

// Encode URI-encoded string value elements in a JSON object
fn encode_elements(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        for (_, v) in obj.iter_mut() {
            if let Some(s) = v.as_str() {
                *v = serde_json::Value::String(
                    form_urlencoded::byte_serialize(s.as_bytes()).collect(),
                );
            }
        }
    }
}

// Extract JSON after checking the Content-Type header
async fn extract_json<T: hyper::Body + Send + 'static>(
    msg: http::Response<T>,
    gate: &slog::Logger,
) -> Result<serde_json::Value, http::Error> {
    if let Some(content_type) = msg.headers().get(header::CONTENT_TYPE) {
        if let Ok(mime_type) = content_type.to_str() {
            if mime_type == "application/json" {
                return msg
                    .into_body()
                    .try_concat()
                    .await
                    .map(|chunk| serde_json::from_slice(&chunk).unwrap_or_default());
            } else {
                slog::warn!(gate, "Incorrect Content-Type: {}", mime_type);
                return Err(http::Error::new(
                    http::ErrorKind::InvalidInput,
                    format!("Incorrect Content-Type: {}", mime_type),
                ));
            }
        }
    }

    // Missing Content-Type
    slog::warn!(gate, "Missing Content-Type: should be application/json");
    msg.into_body()
        .try_concat()
        .await
        .map(|chunk| serde_json::from_slice(&chunk).unwrap_or_default())
}

// Add the NMOS-specific CORS response headers
fn add_cors_preflight_headers(
    req: &http::Request<hyper::Body>,
    mut res: http::Response<hyper::Body>,
) -> http::Response<hyper::Body> {
    if let Some(methods) = res.headers().get(header::ALLOW) {
        res.headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_METHODS, methods.clone());
        res.headers_mut().remove(header::ALLOW);
    }

    if let Some(headers) = req.headers().get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
        res.headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_HEADERS, headers.clone());
    }

    res.headers_mut()
        .insert(header::ACCESS_CONTROL_MAX_AGE, "86400".parse().unwrap());

    res
}

// Map from a resourceType, i.e the plural string used in the API endpoint routes, to a "proper"
// type.
fn type_from_resource_type(resource_type: &str) -> Result<types::Type, &'static str> {
    match resource_type {
        "self" => Ok(types::Type::Node), // for the node API
        "nodes" => Ok(types::Type::Node),
        "devices" => Ok(types::Type::Device),
        "sources" => Ok(types::Type::Source),
        "flows" => Ok(types::Type::Flow),
        "senders" => Ok(types::Type::Sender),
        "receivers" => Ok(types::Type::Receiver),
        "subscriptions" => Ok(types::Type::Subscription),
        "inputs" => Ok(types::Type::Input),
        "outputs" => Ok(types::Type::Output),
        "nc_block" => Ok(types::Type::NcBlock),
        "nc_worker" => Ok(types::Type::NcWorker),
        "nc_manager" => Ok(types::Type::NcManager),
        "nc_device_manager" => Ok(types::Type::NcDeviceManager),
        "nc_class_manager" => Ok(types::Type::NcClassManager),
        "nc_receiver_monitor" => Ok(types::Type::NcReceiverMonitor),
        "nc_receiver_monitor_protected" => Ok(types::Type::NcReceiverMonitorProtected),
        "nc_ident_beacon" => Ok(types::Type::NcIdentBeacon),
        _ => Err("Unknown resource type"),
    }
}

// Map from a "proper" type to a ResourceType, i.e the plural string used in the API endpoint
// routes.
fn resource_type_from_type(type_: types::Type) -> &'static str {
    match type_ {
        types::Type::Node => "nodes",
        types::Type::Device => "devices",
        types::Type::Source => "sources",
        types::Type::Flow => "flows",
        types::Type::Sender => "senders",
        types::Type::Receiver => "receivers",
        types::Type::Subscription => "subscriptions",
        types::Type::Input => "inputs",
        types::Type::Output => "outputs",
        types::Type::NcBlock => "nc_block",
        types::Type::NcWorker => "nc_worker",
        types::Type::NcManager => "nc_manager",
        types::Type::NcDeviceManager => "nc_device_manager",
        types::Type::NcClassManager => "nc_class_manager",
        types::Type::NcReceiverMonitor => "nc_receiver_monitor",
        types::Type::NcReceiverMonitorProtected => "nc_receiver_monitor_protected",
        types::Type::NcIdentBeacon => "nc_ident_beacon",
    }
}

// Construct a standard NMOS "child resources" respnose, from the specified sub-routes
// merging with ones from an existing response
fn make_sub_routes_body(
    sub_routes: HashSet<String>,
    req: &http::Request<hyper::Body>,
    res: http::Response<hyper::Body>,
) -> Result<serde_json::Value, http::Error> {
    let mut results: HashSet<serde_json::Value> = HashSet::new();

    if let Some(body) = res.into_body().chunks().next() {
        if let Ok(body) = body {
            if let Ok(body) = serde_json::from_slice::<Vec<serde_json::Value>>(&body) {
                results.extend(body.into_iter());
            }
        }
    }

    // Experimental extension, to support human-readable HTML rendering of NMOS responses
    if let Some(accept) = req.headers().get(header::ACCEPT) {
        let accept = accept.to_str().unwrap_or_default();
        if accept.contains("text/html") {
            for sub_route in sub_routes {
                results.insert(
                    json!({"$href": format!("{}{}", req.uri(), sub_route), "$_": sub_route}),
                );
            }
        }
    }

    serde_json::to_value(results).mapp_err(|e| http::Error::new(http::ErrorKind::Other, e))
}

// Construct sub-routes for the specified API versions
fn make_api_version_sub_routes(versions: &HashSet<api_version::ApiVersion>) -> HashSet<String> {
    versions.iter().map(|v| make_api_version(v)).collect()
}

// Construct a standard NMOS error response, using the default reason phrase if no user error
// information is specified.
fn make_error_response_body(
    code: StatusCode,
    error: &str,
    debug: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "code": code.as_u16(),
        "error": error,
        "debug": debug.unwrap_or_default(),
    });

    serde_json::to_value(&body).unwrap();
}

mod experimental {
    use http::{header, StatusCode};
    use serde_json::Value;
    use slog::Logger;
    use std::collections::HashMap;
    use std::gmt::Write as _;
    use std::str::FromStr;
    use url::Url;

    const HEADERS_STYLESHEET: &str = r"-stylesheet-(
        .headers {
            font-family: monospace;
            color: grey;
            border-bottom: 1px solid lightgrey;
        }
        .headers ol {
            list-style: none;
            padding: 0;
        }
    )-stylesheet-";

    // Objects with the keywords $href and $_ and rendered as HTML anchor(a) tags.
    // This allows elements in NMOS "child resources" responses to be made into links,
    // and id values in resources can also be made into links to the appropriate resource.
    struct HtmlVisitor<'a, W> {
        writer: W,
        names: HashMap<&'static str, &'a str>,
    }

    impl<'a, W: Write> HtmlVisitor<'a, W> {
        fn new(writer: W) -> Self {
            let mut names = HashMap::new();
            names.insert("http", "http://");
            names.insert("https", "https://");
            names.insert("ws", "ws://");
            names.insert("wss", "wss://");
            HtmlVisitor { writer, names }
        }

        fn write_href(&mut self, href: &str) -> std::fmt::Result {
            let is_href = self.is_href(href);
            self.writer.write_char('"')?;
            if is_href {
                self.writer.write_str("<a href=\"")?;
            }
            self.writer.write_str(href)?;
            if is_href {
                self.writer.write_str("\">")?;
            }
            self.writer.write_char('"');
            Ok(())
        }

        fn is_href(&self, href: &str) -> bool {
            self.names
                .iter()
                .any(|(_, scheme)| href.starts_with(scheme))
        }
    }

    // Construct an HTML rendering of an NMOS response
    pub fn make_html_response_body(res: &http::Response<Vec<u8>>, gate: &Logger) -> String {
        let mut html = String::new();
        write!(&mut html, "<html><head>").unwrap();
        write!(&mut html, "<style>{}</style>", HEADERS_STYLESHEET).unwrap();
        write!(&mut html, "</head><body>").unwrap();
        write!(&mut html, "<div class=\"headers\"><ol>").unwrap();
        for (header_namem, healder_value) in res.headers() {
            write!(
                &mut html,
                "<li><span class=\"name\">{}</span>: <span class=\"value\">",
                header_name.as_str()
            )
            .unwrap();
            if header_name == header::LOCATION {
                let html_value = html_escape(header_value.to_str().unwrap_or_default());
                write!(&mut html, "<a href=\"{}\">{}</a>", html_value, html_value).unwrap();
            } else if header_name == "Link" {
                for (link, rel) in parse_links(header_value.to_str().unwrap_or_default()) {
                    let html_link = html_escape(&link);
                    let html_rel = html_escape(&rel);
                    write!(
                        &mut html,
                        "<{}<a href=\"{}\" rel=\"{}\">{}</a>",
                        html_link, html_link, html_link, html_rel,
                    )
                    .unwrap();
                }
            } else {
                write!(
                    &mut html,
                    "{}",
                    html_escape(header_value.to_str().unwrap_or_default())
                )
                .unwrap();
            }
            write!(&mut html, "</span></li>").unwrap();
        }
        write!(&mut html, "</ol></div><br/>").unwrap();
        write!(&mut html, "<div class=\"json gutter\">").unwrap();
        // Here you should include the JSON body of the response
        //
        // For now, we just include a placeholder
        write!(&mut html, "</div>").unwrap();
        write!(&mut html, "</body></html>").unwrap();
        html
    }

    fn html_escape(text: &str) -> String {
        text.chars()
            .flat_map(|c| match c {
                '<' => "&lt;".chars(),
                '>' => "&gt;".chars(),
                '&' => "&amp;".chars(),
                '"' => "&quot;".chars(),
                '\'' => "&apos;".chars(),
                _ => Some(c),
            })
            .collect()
    }

    fn parse_links(header_value: &str) -> Vec<(String, String)> {
        let mut links = Vec::new();
        for link_rel in header_value.split(", ") {
            if let Some((link, rel)) = parse_link_rel(link_rel) {
                links.push((link, rel));
            }
        }
        links
    }

    fn parse_link_rel(link_rel: &str) -> Option<(String, String)> {
        let mut parts = link_rel.split(": ");
        if let (Some(link), Some(rel)) = (parts.next(), parts.next()) {
            if link.starts_with('<')
                && link.ends_with('>')
                && rel.starts_with("rel=\"")
                && rel.ends_with('"')
            {
                let link = link[1..link.len() - 1].to_string();
                let rel = rel[5..rel.len() - 1].to_string();
                return Some((link, rel));
            }
        }
        None
    }
}
