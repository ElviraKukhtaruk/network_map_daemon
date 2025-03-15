#[cfg(test)]
mod stats_tests {
    use crate::db::schema::Stat;
    use crate::interface::get_stats::save_stat;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use tokio::sync::Mutex;


    #[test]
    fn test_save_stat() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Setup
            let last_stats = Arc::new(Mutex::new(HashMap::<String, Option<Stat>>::new()));

            // Create current stats
            let current_stats = vec![
                Stat {
                    server_id: "test-server".to_string(),
                    interface: "eth0".to_string(),
                    timestamp: 1000,
                    rx: 2000,
                    tx: 3000,
                    rx_p: 200,
                    tx_p: 300,
                    rx_d: 20,
                    tx_d: 30,
                    rx_e: 2,
                    tx_e: 3
                }
            ];

            let result1 = save_stat(Arc::clone(&last_stats), current_stats.clone()).await;
            assert!(result1.is_some());
            let final_stats1 = result1.unwrap();
            assert_eq!(final_stats1.len(), 0);

            let updated_stats = vec![
                Stat {
                    server_id: "test-server".to_string(),
                    interface: "eth0".to_string(),
                    timestamp: 1001,
                    rx: 2500,    // +500
                    tx: 3800,    // +800
                    rx_p: 250,   // +50
                    tx_p: 380,   // +80
                    rx_d: 25,    // +5
                    tx_d: 38,    // +8
                    rx_e: 4,     // +2
                    tx_e: 6      // +3
                }
            ];

            // Second call should produce stats with differences
            let result2 = save_stat(Arc::clone(&last_stats), updated_stats).await;
            assert!(result2.is_some());
            let final_stats2 = result2.unwrap();
            assert_eq!(final_stats2.len(), 1);

            // Check that the difference calculations are correct
            let diff_stat = &final_stats2[0];
            assert_eq!(diff_stat.server_id, "test-server");
            assert_eq!(diff_stat.interface, "eth0");
            assert_eq!(diff_stat.timestamp, 1001);
            assert_eq!(diff_stat.rx, 500);  // 2500 - 2000
            assert_eq!(diff_stat.tx, 800);  // 3800 - 3000
            assert_eq!(diff_stat.rx_p, 50); // 250 - 200
            assert_eq!(diff_stat.tx_p, 80); // 380 - 300
            assert_eq!(diff_stat.rx_d, 5);  // 25 - 20
            assert_eq!(diff_stat.tx_d, 8);  // 38 - 30
            assert_eq!(diff_stat.rx_e, 2);  // 4 - 2
            assert_eq!(diff_stat.tx_e, 3);  // 6 - 3
        });
    }

    #[test]
    fn test_save_stat_multiple_interfaces() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let last_stats = Arc::new(Mutex::new(HashMap::<String, Option<Stat>>::new()));

            let initial_stats = vec![
                Stat {
                    server_id: "test-server".to_string(),
                    interface: "eth0".to_string(),
                    timestamp: 1000,
                    rx: 1000,
                    tx: 2000,
                    rx_p: 100,
                    tx_p: 200,
                    rx_d: 10,
                    tx_d: 20,
                    rx_e: 1,
                    tx_e: 2
                },
                Stat {
                    server_id: "test-server".to_string(),
                    interface: "eth1".to_string(),
                    timestamp: 1000,
                    rx: 3000,
                    tx: 4000,
                    rx_p: 300,
                    tx_p: 400,
                    rx_d: 30,
                    tx_d: 40,
                    rx_e: 3,
                    tx_e: 4
                }
            ];

            let _ = save_stat(Arc::clone(&last_stats), initial_stats).await;
            let updated_stats = vec![
                Stat {
                    server_id: "test-server".to_string(),
                    interface: "eth0".to_string(),
                    timestamp: 1001,
                    rx: 1500,    // +500
                    tx: 2800,    // +800
                    rx_p: 150,   // +50
                    tx_p: 280,   // +80
                    rx_d: 15,    // +5
                    tx_d: 28,    // +8
                    rx_e: 3,     // +2
                    tx_e: 5      // +3
                },
                Stat {
                    server_id: "test-server".to_string(),
                    interface: "eth1".to_string(),
                    timestamp: 1001,
                    rx: 3100,    // +100
                    tx: 4200,    // +200
                    rx_p: 310,   // +10
                    tx_p: 420,   // +20
                    rx_d: 31,    // +1
                    tx_d: 42,    // +2
                    rx_e: 4,     // +1
                    tx_e: 6      // +2
                }
            ];

            let result = save_stat(Arc::clone(&last_stats), updated_stats).await;
            assert!(result.is_some());
            let final_stats = result.unwrap();
            assert_eq!(final_stats.len(), 2);

            let mut sorted_stats = final_stats;
            sorted_stats.sort_by(|a, b| a.interface.cmp(&b.interface));

            let eth0_diff = &sorted_stats[0];
            assert_eq!(eth0_diff.interface, "eth0");
            assert_eq!(eth0_diff.rx, 500);
            assert_eq!(eth0_diff.tx, 800);

            let eth1_diff = &sorted_stats[1];
            assert_eq!(eth1_diff.interface, "eth1");
            assert_eq!(eth1_diff.rx, 100);
            assert_eq!(eth1_diff.tx, 200);
        });
    }
}
