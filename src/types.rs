use std::collections::HashMap;

// Define a module for NMOS
mod nmos {
    // Define a string enum for resource types
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum ResourceType {
        Node,
        Device,
        Source,
        Flow,
        Sender,
        Receiver,
        Subscription,
        Grain,
        Input,
        Output,
        Global,
        NcBlock,
        NcWorker,
        NcManager,
        NcDeviceManager,
        NcClassManager,
        NcReceieverMonitor,
        NcReceieverMonitorProtected,
        NcIdentBeacon,
    }

    // Define a constant hashmap for resource types
    lazy_static::lazy_static! {
        pub static ref ALL_TYPES: HashMap<ResourceType, &'static str> = {
            let mut map = HashMap::new();
            map.insert(ResourceType::Node, "node");
            map.insert(ResourceType::Device, "device");
            map.insert(ResourceType::Source, "source");
            map.insert(ResourceType::Flow, "flow");
            map.insert(ResourceType::Sender, "sender");
            map.insert(ResourceType::Receiver, "receiver");
            map.insert(ResourceType::Subscription, "subscription");
            map.insert(ResourceType::Grain, "grain");
            map.insert(ResourceType::Input, "input");
            map.insert(ResourceType::Output, "output");
            map.insert(ResourceType::Global, "global");
            map.insert(ResourceType::NcBlock, "nc_block");
            map.insert(ResourceType::NcWorker, "nc_worker");
            map.insert(ResourceType::NcManager, "nc_manager");
            map.insert(ResourceType::NcDeviceManager, "nc_device_manager");
            map.insert(ResourceType::NcClassManager, "nc_class_manager");
            map.insert(ResourceType::NcReceieverMonitor, "nc_receiver_monitor");
            map.insert(ResourceType::NcReceieverMonitorProtected, "nc_receiver_monitor_protected");
            map.insert(ResourceType::NcIdentBeacon, "nc_ident_beacon");
            map
        };
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_enum() {
        assert_eq!(nmos::ResourceType::Node, nmos::ResourceType::Node);
        assert_ne!(nmos::ResourceType::Node, nmos::ResourceType::Device);
    }

    #[test]
    fn test_all_types() {
        assert_eq!(nmos::ALL_TYPES.len(), 19);
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Node], "node");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Device], "device");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Source], "source");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Flow], "flow");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Sender], "sender");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Receiver], "receiver");
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::Subscription],
            "subscription"
        );
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Grain], "grain");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Input], "input");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Output], "output");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::Global], "global");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::NcBlock], "nc_block");
        assert_eq!(nmos::ALL_TYPES[&nmos::ResourceType::NcWorker], "nc_worker");
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::NcManager],
            "nc_manager"
        );
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::NcDeviceManager],
            "nc_device_manager"
        );
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::NcClassManager],
            "nc_class_manager"
        );
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::NcReceieverMonitor],
            "nc_receiver_monitor"
        );
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::NcReceieverMonitorProtected],
            "nc_receiver_monitor_protected"
        );
        assert_eq!(
            nmos::ALL_TYPES[&nmos::ResourceType::NcIdentBeacon],
            "nc_ident_beacon"
        );
    }
}
