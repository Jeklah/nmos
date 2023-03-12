use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::{json, Value};
use slog::{info, Logger};
use nmos::node_model;
use nmos::read_lock;
use nmos::write_lock;
use nmos::fields::{self, id, type_};
use nmos::{self, details, node_model};
use web_sys::Json;

use crate::nmos::{self, ActivationMode, ActivationState, JSONFields, Model, Tai};
use crate::rest::basic_utils::as_tai;
use crate::rest::error::{Error, Result};

// Construct a 'not pending' activation response object with all null values
pub fn make_activation() -> Value {
    json!({
        JSONFields::Mode: Value::Null,
        JSONFields::RequestedTime: Value::Null,
        JSONFields::ActivationTime: Value::Null,
    })
}

// Discover which kind of activation this is, or whether it is only a request for staging
pub fn get_activation_state(activation: &Value) -> Result<ActivationState> {
    if activation.is_null() {
        return Ok(ActivationState::StagingOnly);
    }

    let mode_or_null = nmos::fields::mode(activation)?;
    if mode_or_null.is_null() {
        return Ok(ActivationState::ActivationNotPending);

    }

    let mode = ActivationMode::from_str(mode_or_null.as_str().unwrap())?;

    match mode {
        ActivationMode::ActivateScheduledAbsolute | ActivationMode::ActivateScheduledRelative => Ok(ActivationState::ScheduledActivationPending),
        ActivationMode::ActivateImmediate => Ok(ActivationState::ImmediateActivationPending),
        _ => Err(Error::new("invalid activation mode")),
    }
}

// Calculate the absolute TAI from the requested time of a scheduled activation
pub fn get_absolute_requested_time(activation: &Value, request_time: &DateTime<Utc>, logger: &Logger) -> Result<Tai> {
    let mode_or_null = nmos::fields::mode(activation)?;
    let requested_time_or_null = nmos::fields::requested_time(activation)?;

    let mode = ActivationMode::from_str(mode_or_null.as_str().unwrap())?;

    match mode {
        ActivationMode::ActivateScheduledAbsolute => {
            let requested_time = as_tai(&requested_time_or_null)?;
            Ok(requested_time)
        }
        ActivationMode::ActivateScheduledRelative => {
            let duration = Duration::from_std(as_tai(&requested_time_or_null)?.as_duration())?;
            let requested_time = request_time + duration;
            Ok(Tai::from(requested_time))
        }
        _ => Err(Error::new(format!("cannot get absolute requested time for mode: {}", mode))),
    }
}



// Set the appropriate fields of the response/staged activation from the specified request
fn merge_activation(activation: &mut Json, request_activation: &Json, request_time: &nmos::tai) {
    match details::get_activation_state(request_activation) {
        details::ActivationState::StagingOnly => {
            // All three merged values should be null (already)
            activation[nmos::fields::MODE] = Json::Null;
            activation[nmos::fields::REQUESTED_TIME] = Json::Null;

            // "If no activation was requested in the PATCH `activation_time` will be set `null`."
            // See https://specs.amwa.tv/is-05/releases/v1.0.0/APIs/ConnectionAPI.html
            activation[nmos::fields::ACTIVATION_TIME] = Json::Null;
        }
        details::ActivationState::ActivationNotPending => {
            // Merged "mode" should be null (already)
            activation[nmos::fields::MODE] = Json::Null;

            // Each of these fields "returns to null [...] when the resource is unlocked by setting the activation mode to null."
            // See https://specs.amwa.tv/is-05/releases/v1.0.0/APIs/schemas/with-refs/v1.0-activation-response-schema.html
            // and https://specs.amwa.tv/is-05/releases/v1.1.0/APIs/schemas/with-refs/activation-response-schema.html
            activation[nmos::fields::REQUESTED_TIME] = Json::Null;
            activation[nmos::fields::ACTIVATION_TIME] = Json::Null;
        }
        details::ActivationState::ImmediateActivationPending => {
            // Merged "mode" should be "activate_immediate", and "requested_time" should be null (already)
            activation[nmos::fields::MODE] = Json::String(nmos::activation_modes::ACTIVATE_IMMEDIATE.name);

            // "For an immediate activation this field will always be null on the staged endpoint,
            // even in the response to the PATCH request."
            // However, here it is set to indicate an in-flight immediate activation
            activation[nmos::fields::REQUESTED_TIME] = Json::String(nmos::make_version(request_time));

            // "For immediate activations on the staged endpoint this property will be the time the activation actually
            // occurred in the response to the PATCH request, but null in response to any GET requests thereafter."
            // Therefore, this value will be set later
            activation[nmos::fields::ACTIVATION_TIME] = Json::Null;
        }
        details::ActivationState::ScheduledActivationPending => {
            // Merged "mode" and "requested_time" should be set (already)
            activation[nmos::fields::MODE] = request_activation.at(nmos::fields::MODE).unwrap();
            activation[nmos::fields::REQUESTED_TIME] = request_activation.at(nmos::fields::REQUESTED_TIME).unwrap();

            // "For scheduled activations `activation_time` should be the absolute TAI time the parameters will actually transition."
            // See https://specs.amwa.tv/is-05/releases/v1.0.0/APIs/ConnectionAPI.html
            let absolute_requested_time = get_absolute_requested_time(&activation, request_time);
            activation[nmos::fields::ACTIVATION_TIME] = Json::String(nmos::make_version(&absolute_requested_time));
        }
    }
}

// This is a bit of a dirty hack to support both Connection API and Channel Mapping API
// without passing additional arguments around.
fn get_resources_for_type(model: &mut nmos::node_model, type_: nmos::type) -> &mut nmos::resources {
if type_ == nmos::types::input || type_ == nmos::types::output {
&mut model.channelmapping_resources
} else {
&mut model.connection_resources
}
}

struct ImmediateActivationNotPending<'a> {
model: &'a nmos::node_model,
id_type: (nmos::id, nmos::type),
}

impl<'a> ImmediateActivationNotPending<'a> {
fn new(model: &'a nmos::node_model, id_type: (nmos::id, nmos::type)) -> Self {
Self { model, id_type }
}

fn operator(&self) -> bool {
    if self.model.shutdown {
        return true;
    }

    let resources = get_resources_for_type(self.model, self.id_type.1);

    let resource = find_resource(resources, self.id_type);
    if resource.is_none() {
        return true;
    }

    let mode = nmos::fields::mode(nmos::fields::activation(nmos::fields::endpoint_staged(
        resource.unwrap().data,
    )));
    return mode.is_null() || mode.as_string() != nmos::activation_modes::activate_immediate.name;
}



fn wait_immediate_activation_not_pending(model: &mut nmos::node_model, lock: &mut read_lock, id_type: &(id::Id, type_::Type)) -> bool {
    model.wait_for(lock, std::time::Duration::from_secs(fields::immediate_activation_max(model.settings)), immediate_activation_not_pending(model, id_type))
}

fn wait_immediate_activation_not_pending(model: &mut nmos::node_model, lock: &mut write_lock, id_type: &(id::Id, type_::Type)) -> bool {
    model.wait_for(lock, std::time::Duration::from_secs(fields::immediate_activation_max(model.settings)), immediate_activation_not_pending(model, id_type))
}

fn wait_activation_modified(model: &mut nmos::node_model, lock: &mut write_lock, id_type: (id::Id, type_::Type), initial_activation: web::json::JsonValue) -> bool {
    model.wait_for(lock, std::time::Duration::from_secs(fields::immediate_activation_max(model.settings)), || {
        if model.shutdown {
            return true;
        }

        let resources = get_resources_for_type(model, id_type.1);
        let resource = find_resource(resources, id_type);
        if resource.is_none() {
            return true;
        }

        let activation = &fields::activation(fields::endpoint_staged(resource.unwrap().data));
        activation != &initial_activation
    })
}

fn handle_immediate_activation_pending(model: &mut nmos::node_model, lock: &mut write_lock, id_type: &(id::Id, type_::Type), response_activation: &mut web::json::JsonValue, gate: &mut slog::Logger) {
    if !wait_activation_modified(model, lock, *id_type, response_activation.clone()) || model.shutdown {
        panic!("timed out waiting for in-flight immediate activation to complete");
    }

    let resources = get_resources_for_type(model, id_type.1);
    let found = find_resource(resources, *id_type).unwrap();
    if found.is_none() {
        panic!("resource vanished during in-flight immediate activation");
    } else {
        let staged = &mut fields::endpoint_staged(found.unwrap().data);
        let staged_activation = &mut staged[fields::activation];

        if staged_activation[fields::requested_time] != response_activation[fields::requested_time] {
            panic!("activation modified during in-flight immediate activation");
        }

        modify_resource(resources, id_type.0, |resource| {
            let staged = &mut fields::endpoint_staged(resource.data);
            let staged_activation = &mut staged[fields::activation];

            resource.data[fields::version] = web::json::JsonValue::from(nmos::make_version());

            staged_activation[fields::mode] = web::json::JsonValue::Null;
            response_activation[fields::requested_time] = web::json::JsonValue::Null;
            staged_activation[fields::requested_time] = web::json::JsonValue::Null;

            response_activation[fields::activation_time] = staged_activation[fields::activation_time].take();
            staged_activation[fields::activation_time] = web::json::JsonValue::Null;
        });

        slog::info!(gate, "Notifying API - immediate activation completed");
        model.notify();
    }
}
}
fn downgrade(resource: &nmos::Resource, version: &nmos::ApiVersion) -> web::json::JsonValue {
    downgrade_with(resource, version, version)
}

fn downgrade_with(resource: &nmos::Resource, version: &nmos::ApiVersion, downgrade_version: &nmos::ApiVersion) -> web::json::JsonValue {
    downgrade(resource.version, resource.downgrade_version, resource.type, resource.data.clone(), version, downgrade_version);
}

use std::collections::HashMap;

static RESOURCES_VERSIONS: &'static HashMap<nmos::type, HashMap<nmos::api_version, Vec<utility::string_t>>> = &{
    let mut map = HashMap::new();
    map.insert(nmos::types::node, {
        let mut node_versions = HashMap::new();
        node_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("version"), U("label"), U("href"), U("hostname"), U("caps"), U("services")]);
        node_versions.insert(nmos::is04_versions::v1_1, vec![U("description"), U("tags"), U("api"), U("clocks")]);
        node_versions.insert(nmos::is04_versions::v1_2, vec![U("interfaces")]);
        node_versions
    });
    map.insert(nmos::types::device, {
        let mut device_versions = HashMap::new();
        device_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("version"), U("label"), U("type"), U("node_id"), U("senders"), U("receivers")]);
        device_versions.insert(nmos::is04_versions::v1_1, vec![U("description"), U("tags"), U("controls")]);
        device_versions
    });
    map.insert(nmos::types::source, {
        let mut source_versions = HashMap::new();
        source_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("version"), U("label"), U("description"), U("format"), U("caps"), U("tags"), U("device_id"), U("parents")]);
        source_versions.insert(nmos::is04_versions::v1_1, vec![U("grain_rate"), U("clock_name"), U("channels")]);
        source_versions.insert(nmos::is04_versions::v1_3, vec![U("event_type")]);
        source_versions
    });
    map.insert(nmos::types::flow, {
        let mut flow_versions = HashMap::new();
        flow_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("version"), U("label"), U("description"), U("format"), U("tags"), U("source_id"), U("parents")]);
        flow_versions.insert(nmos::is04_versions::v1_1, vec![U("grain_rate"), U("device_id"), U("media_type"), U("sample_rate"), U("bit_depth"), U("DID_SDID"), U("frame_width"), U("frame_height"), U("interlace_mode"), U("colorspace"), U("transfer_characteristic"), U("components")]);
        flow_versions.insert(nmos::is04_versions::v1_3, vec![U("event_type")]);
        flow_versions
    });
    map.insert(nmos::types::sender, {
        let mut sender_versions = HashMap::new();
        sender_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("version"), U("label"), U("description"), U("flow_id"), U("transport"), U("tags"), U("device_id"), U("manifest_href")]);
        sender_versions.insert(nmos::is04_versions::v1_2, vec![U("caps"), U("interface_bindings"), U("subscription")]);
        sender_versions
    });
    map.insert(nmos::types::receiver, {
        let mut receiver_versions = HashMap::new();
        receiver_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("version"), U("label"), U("description"), U("format"), U("caps"), U("tags"), U("device_id"), U("transport"), U("subscription"))];
        receiver_versions.insert(nmos::is04_versions::v1_2, vec![U("interface_bindings")]);
        receiver_versions
    });
    map.insert(nmps::types::subscription, {
        let mut subscription_versions = HashMap::new();
        subscription_versions.insert(nmos::is04_versions::v1_0, vec![U("id"), U("ws_href"), U("max_updates_rate_ms"), U("persist"), U("resource_path"), U("params")]);
        subscription_versions.insert(nmos::is04_versions::v1_1, vec![U("secure")]);
        subscription_versions.insert(nmos::is04_versions::v1_3, vec![U("authorization")]);
        subscription_versions
    });
    resources_versions;
};
