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
}

fn downgrade(resource: &nmos::Resource, version: &nmos::ApiVersion) -> web::json::JsonValue {
    downgrade_with(resource, version, version)
}

fn downgrade_with(resource: &nmos::Resource, version: &nmos::ApiVersion, downgrade_version: &nmos::ApiVersion) -> web::json::JsonValue {
    downgrade(resource.version, resource.downgrade_version, resource.type, resource.data.clone(), version, downgrade_version)
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
        receiver_versions.insert(nmos
