#[cfg(test)]
mod tests {
    use crate::interface::info::{get_interface_name_from_attribute, get_loopback_from_header};
    use std::net::Ipv6Addr;
    use netlink_packet_route::link::{LinkAttribute, LinkFlag, LinkHeader, LinkLayerType};

    #[test]
    fn test_get_interface_name_from_attribute() {
        let attributes = vec![
            LinkAttribute::IfName("eth0".to_string()),
            LinkAttribute::Mtu(1500),
        ];

        let name = get_interface_name_from_attribute(attributes);
        assert_eq!(name, Some("eth0".to_string()));
    }

    #[test]
    fn test_get_interface_name_from_attribute_none() {
        let attributes = vec![
            LinkAttribute::Mtu(1500),
        ];

        let name = get_interface_name_from_attribute(attributes);
        assert_eq!(name, None);
    }

    #[test]
    fn test_get_loopback_from_header() {
        // Test loopback interface
        let mut flags = Vec::new();
        flags.push(LinkFlag::Loopback);

        let header = LinkHeader {
            index: 1,
            link_layer_type: LinkLayerType::Ether,
            interface_family: netlink_packet_route::AddressFamily::Inet,
            flags: flags.into_iter().collect(),
            change_mask: Default::default(),
        };

        assert!(get_loopback_from_header(header));

        // Test non-loopback interface
        let header = LinkHeader {
            index: 2,
            link_layer_type: LinkLayerType::Ether,
            interface_family: netlink_packet_route::AddressFamily::Inet,
            flags: vec![LinkFlag::Up].into_iter().collect(),
            change_mask: Default::default(),
        };

        assert!(!get_loopback_from_header(header));
    }

    #[test]
    fn test_compare_addresses() {
        use crate::db::schema::Addr;
        use crate::interface::get_address::compare;

        // Create test addresses
        let addr1 = Addr {
            server_id: "test".to_string(),
            interface: "eth0".to_string(),
            ipv6: vec![(Some(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), Some(128))],
            ipv6_peer: vec![]
        };

        let addr2 = Addr {
            server_id: "test".to_string(),
            interface: "eth1".to_string(),
            ipv6: vec![(Some(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2)), Some(128))],
            ipv6_peer: vec![]
        };

        // Same as addr1 but different IP
        let addr3 = Addr {
            server_id: "test".to_string(),
            interface: "eth0".to_string(),
            ipv6: vec![(Some(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 3)), Some(128))],
            ipv6_peer: vec![]
        };

        // Test updates
        let fresh = vec![addr1.clone(), addr2.clone()];
        let db = vec![addr3.clone(), addr2.clone()];

        let diff = compare(&fresh, &db);

        assert_eq!(diff.updates.len(), 1);
        assert_eq!(diff.updates[0].interface, "eth0");
        assert_eq!(diff.creates.len(), 0);
        assert_eq!(diff.deletes.len(), 0);

        // Test creates
        let fresh = vec![addr1.clone(), addr2.clone()];
        let db = vec![addr2.clone()];

        let diff = compare(&fresh, &db);

        assert_eq!(diff.updates.len(), 0);
        assert_eq!(diff.creates.len(), 1);
        assert_eq!(diff.creates[0].interface, "eth0");
        assert_eq!(diff.deletes.len(), 0);

        // Test deletes
        let fresh = vec![addr1.clone()];
        let db = vec![addr1.clone(), addr2.clone()];

        let diff = compare(&fresh, &db);

        assert_eq!(diff.updates.len(), 0);
        assert_eq!(diff.creates.len(), 0);
        assert_eq!(diff.deletes.len(), 1);
        assert_eq!(diff.deletes[0].interface, "eth1");
    }

    #[test]
    fn test_filter_empty_updates() {
        use crate::db::schema::Addr;
        use crate::interface::get_address::compare;

            // Address with empty IP arrays
        let empty_addr = Addr {
            server_id: "test".to_string(),
            interface: "eth0".to_string(),
            ipv6: vec![],
            ipv6_peer: vec![]
        };

        // Address with valid IP
        let valid_addr = Addr {
            server_id: "test".to_string(),
            interface: "eth0".to_string(),
            ipv6: vec![(Some(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), Some(128))],
            ipv6_peer: vec![]
        };

        // Test what happens when fresh data has empty IPs
        let fresh = vec![empty_addr.clone()];
        let db = vec![valid_addr.clone()];

        let diff = compare(&fresh, &db);
        assert_eq!(diff.updates.len(), 1);

        assert!(diff.updates[0].ipv6.is_empty());

        let filtered_updates = diff.updates.into_iter()
            .filter(|addr| !addr.ipv6.is_empty())
            .collect::<Vec<_>>();

        // After filtering, there should be no updates
        assert_eq!(filtered_updates.len(), 0);
    }

    #[test]
    fn test_handle_wlp1s0_edge_case() {
        use crate::db::schema::Addr;
        use crate::interface::get_address::compare;

        let wlp1s0_empty = Addr {
            server_id: "2420549211b547559bef4ab3e5e25571".to_string(),
            interface: "wlp1s0".to_string(),
            ipv6: vec![],
            ipv6_peer: vec![]
        };

        let wlp1s0_with_ip = Addr {
            server_id: "2420549211b547559bef4ab3e5e25571".to_string(),
            interface: "wlp1s0".to_string(),
            ipv6: vec![(Some(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)), Some(64))],
            ipv6_peer: vec![]
        };
        let fresh = vec![wlp1s0_empty.clone()];
        let db = vec![wlp1s0_with_ip.clone()];

        let diff = compare(&fresh, &db);

        assert_eq!(diff.updates.len(), 1);
        assert_eq!(diff.updates[0].interface, "wlp1s0");
        assert!(diff.updates[0].ipv6.is_empty());

        let valid_updates = diff.updates.into_iter()
            .filter(|addr| !addr.ipv6.is_empty())
            .collect::<Vec<_>>();

        assert_eq!(valid_updates.len(), 0);
    }
}
