use nmos::{api_version, is04_versions, resource, types};
use std::collections::HashMap;

// See https://specs.amwa.tv/is-04/releases/v1.2.0/docs/2.5._APIs_-_Query_Parameters.html#downgrade-queries

fn is_permitted_downgrade(resource: &resource, version: &api_version) -> bool {
    is_permitted_downgrade_with_downgrade_version(resource, version, version)
}

fn is_permitted_downgrade_with_downgrade_version(
    resource: &resource,
    version: &api_version,
    downgrade_version: &api_version,
) -> bool {
    is_permitted_downgrade_impl(
        &resource.version,
        &resource.downgrade_version,
        &resource.r#type,
        version,
        downgrade_version,
    )
}

fn is_permitted_downgrade_impl(
    resource_version: &api_version,
    resource_downgrade_version: &api_version,
    resource_type: &types,
    version: &api_version,
    downgrade_version: &api_version,
) -> bool {
    // If the resource has a hard-coded minimum API version, simply don't permit downgrade
    // This avoids e.g. validating the resource against the relevant schema every time!
    if resource_downgrade_version > downgrade_version {
        return false;
    }

    // Enforce that "downgrade queries may not be performed between major API versions."
    if version.major != downgrade_version.major {
        return false;
    }
    if resource_version.major != version.major {
        return false;
    }

    // Only "permit old-versioned responses to be provided to clients which are confident that they can handle any missing attributes between the specified API versions"
    // and never perform an upgrade!
    if resource_version.minor < downgrade_version.minor {
        return false;
    }

    // Finally, "only ever return subscriptions which were created against the same API version".
    // See https://basecamp.com/1791706/projects/10192586/messages/70664054#comment_544216653
    if *resource_type == types::subscription && version != resource_version {
        return false;
    }

    true
}

mod details {
    use nmos::{api_version, make_api_version, resource, types};
    use std::string::String;

    pub fn make_permitted_downgrade_error(
        resource: &resource,
        version: &api_version,
    ) -> String {
        make_permitted_downgrade_error_with_downgrade_version(
            resource,
            version,
            version,
        )
    }

    pub fn make_permitted_downgrade_error_with_downgrade_version(
        resource: &resource,
        version: &api_version,
        downgrade_version: &api_version,
    ) -> String {
        make_permitted_downgrade_error_impl(
            &resource.version,
            &resource.r#type,
            version,
            downgrade_version,
        )
    }

    fn make_permitted_downgrade_error_impl(
        resource_version: &api_version,
        resource_type: &types,
        version: &api_version,
        downgrade_version: &api_version,
    ) -> String {
        if version == downgrade_version {
            make_api_version(version) + " request"
        } else {
            make_api_version(downgrade_version) + " downgrade request"
        } + " is not permitted for a " + &make_api_version(resource_version) + " resource"
    }

    use std::collections::HashMap;

fn downgrade(resource: &nmos::resource, version: &nmos::api_version) -> web::json::Value {
    downgrade(resource, version, version)
}

fn downgrade(
    resource: &nmos::resource,
    version: &nmos::api_version,
    downgrade_version: &nmos::api_version,
) -> web::json::Value {
    downgrade(
        resource.version,
        resource.downgrade_version,
        resource.type_,
        resource.data.clone(),
        version,
        downgrade_version,
    )
}

fn resources_versions() -> &'static HashMap<
    nmos::type_,
    HashMap<nmos::api_version, Vec<utility::string::String>>,
> {
    static RESOURCES_VERSIONS: HashMap<
        nmos::type_,
        HashMap<nmos::api_version, Vec<utility::string::String>>,
    > = {
        let mut map = HashMap::new();
        map.insert(
            nmos::types::node,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "href", "hostname", "caps", "services",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "description", "tags", "api", "clocks",
                ]),
                (nmos::is04_versions::v1_2, vec![
                    "interfaces",
                ]),
            ].into_iter()
            .collect(),
        );
        map.insert(
            nmos::types::device,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "type", "node_id", "senders", "receivers",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "description", "tags", "controls",
                ]),
            ].into_iter()
            .collect(),
        );
        map.insert(
            nmos::types::source,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "description", "format", "caps", "tags", "device_id", "parents",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "grain_rate", "clock_name", "channels",
                ]),
                (nmos::is04_versions::v1_3, vec![
                    "event_type",
                ]),
            ].into_iter()
            .collect(),
        );
        map.insert(
            nmos::types::flow,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "description", "format", "tags", "source_id", "parents",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "grain_rate", "device_id", "media_type", "sample_rate", "bit_depth", "DID_SDID",
                    "frame_width", "frame_height", "interlace_mode", "colorspace", "transfer_characteristic",
                    "components",
                ]),
                (nmos::is04_versions::v1_3, vec![
                    "event_type",
                ]),
            ].into_iter()
            .collect(),
        );
        map.insert(
            nmos::types::sender,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "description", "tags", "device_id", "flow_id", "transport", "manifest_href",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "interface_bindings", "subscription", "master_enable",
                ]),
                (nmos::is04_versions::v1_2, vec![
                    "grain_rate", "master_enable", "activation", "subscription", "interface_bindings",
                ]),
                (nmos::is04_versions::v1_3, vec![
                    "event_type",
                ]),
            ].into_iter()
            .collect(),
        );
        map.insert(
            nmos::types::receiver,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "description", "tags", "device_id", "flow_id", "transport", "subscription",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "interface_bindings", "subscription", "master_enable",
                ]),
                (nmos::is04_versions::v1_2, vec![
                    "grain_rate", "master_enable", "activation", "subscription", "interface_bindings",
                ]),
                (nmos::is04_versions::v1_3, vec![
                    "event_type",
                ]),
            ].into_iter()
            .collect(),
        );
        map.insert(
            nmos::types::subscription,
            vec![
                (nmos::is04_versions::v1_0, vec![
                    "id", "version", "label", "description", "tags", "resource_path", "params",
                ]),
                (nmos::is04_versions::v1_1, vec![
                    "persist", "receiver_id", "sender_id", "active",
                ]),
                (nmos::is04_versions::v1_2, vec![
                    "persist", "receiver_id", "sender_id", "active", "receiver_active", "sender_active",
                ]),
            ].into_iter()
            .collect(),
        );

fn downgrade(
    resource_version: nmos::ApiVersion,
    resource_downgrade_version: nmos::ApiVersion,
    resource_type: nmos::Type,
    resource_data: web::json::Value,
    version: nmos::ApiVersion,
    downgrade_version: nmos::ApiVersion,
) -> web::json::Value {
    if !is_permitted_downgrade(
        resource_version,
        resource_downgrade_version,
        resource_type,
        version,
        downgrade_version,
    ) {
        return web::json::Value::Null;
    }

    // optimisation for no resource data (special case)
    if resource_data.is_null() {
        return resource_data;
    }

    // optimisation for the common case (old-versioned resources, if being permitted, do not get upgraded)
    if resource_version <= version {
        return resource_data;
    }

    let mut result = web::json::Value::Object(Default::default());

    let resource_versions = resources_versions().get(&resource_type).unwrap();
    let version_first = resource_versions.iter();
    let version_last = resource_versions.upper_bound(&version);
    let mut result = json::object::Object::new();
    for version_properties in version_first {
        if version_properties.0 > &version {
            break;
        }
    for property in &version_properties.1 {
        if resource_data.has_key(property) {
            result.insert(property.clone(), resource_data[property].clone());
        }
    }
}
json::JsonValue::Object(result)
