use std::sync::Arc;
use regex::Regex;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value as JsonValue};
use crate::nmos::{ID, Type};
use crate::nmos::mutex::{ReadLock, WriteLock};
use crate::nmos::tai::Tai;
use crate::slog::Logger;

pub enum ActivationState {
    ImmediateActivationPending,
    ScheduledActivationPending,
    ActivationNotPending,
    StagingOnly,
}

pub fn get_activation_state(activation: &JsonValue) -> ActivationState {
    if let Some(true) = activation.get("activation.pending") {
        if let Some(requested_time) = activation.get("activation.requested_time") {
            let requested_time = DateTime::parse_from_rfc3339(requested_time.as_str().unwrap())
                .unwrap()
                .into();
            let now = Utc::now().into();
            if requested_time <= now {
                ActivationState::ImmediateActivationPending
            } else {
                ActivationState::ScheduledActivationPending
            }
        } else {
            ActivationState::ImmediateActivationPending
        }
    } else {
        ActivationState::ActivationNotPending
    }
}

pub fn get_absolute_requested_time(activation: &JsonValue, request_time: Tai) -> Tai {
    let requested_time = activation.get("activation.requested_time").unwrap().as_str().unwrap();
    let requested_time = DateTime::parse_from_rfc3339(requested_time)
        .unwrap()
        .into();
    Tai::from(requested_time + (request_time - request_time.to_rounded_tai()) - request_time.to_rounded_tai())
}

pub fn merge_activation(activation: &mut JsonValue, request_activation: &JsonValue, request_time: Tai) {
    if let Some(true) = request_activation.get("activation.pending") {
        let now = Utc::now().into();
        if let Some(requested_time) = request_activation.get("activation.requested_time") {
            let requested_time = DateTime::parse_from_rfc3339(requested_time.as_str().unwrap())
                .unwrap()
                .into();
            if requested_time > now {
                activation["activation"]["requested_time"] = json!(requested_time.to_rfc3339());
            } else {
                activation["activation"]["requested_time"] = json!(now.to_rfc3339());
            }
        } else {
            activation["activation"]["requested_time"] = json!(now.to_rfc3339());
        }
    }
    activation["activation"]["mode"] = request_activation.get("activation.mode").cloned().unwrap_or(JsonValue::Null);
    activation["activation"]["transport_params"] = request_activation.get("activation.transport_params").cloned().unwrap_or(JsonValue::Null);
    activation["activation"]["persist"] = request_activation.get("activation.persist").cloned().unwrap_or(JsonValue::Null);
}

fn wait_immediate_activation_not_pending(model: &node_model, lock: &read_lock, id_type: &(id, type_)) -> bool {
    // function body
}

fn wait_immediate_activation_not_pending(model: &mut node_model, lock: &write_lock, id_type: &(id, type_)) -> bool {
    // function body
}

fn handle_immediate_activation_pending(model: &mut node_model, lock: &write_lock, id_type: &(id, type_), response_activation: &mut value, gate: &base_gate) {
    // function body
}