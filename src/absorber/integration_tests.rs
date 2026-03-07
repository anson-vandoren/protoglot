#[cfg(test)]
mod tests {
    use std::io::Write;
    use crate::absorber::stats_svc::StatsSvc;
    use crate::config::MessageType;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use crate::absorber::tcp::handle_tcp_connection;

    #[tokio::test]
    async fn test_tcp_absorber_decompression_metrics_direct() {
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;
        
        // Prepare gzipped data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        let original_data = b"{\"key\": \"value\"}\n";
        encoder.write_all(original_data).unwrap();
        let compressed_data = encoder.finish().unwrap();
        
        let compressed_len = compressed_data.len();
        let original_len = original_data.len();
        
        // Mock socket using a cursor
        let socket = std::io::Cursor::new(compressed_data);
        
        // Run handle_tcp_connection
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        
        // Allow some time for the stats task to process messages
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let (events, raw_bytes, decomp_bytes) = stats.get_stats().await;
        
        assert_eq!(events, 1);
        assert_eq!(raw_bytes, compressed_len);
        assert_eq!(decomp_bytes, original_len);
    }

    #[tokio::test]
    async fn test_tcp_absorber_decompression_metrics_with_trailing_newline() {
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;
        
        // Prepare gzipped data with TWO newlines
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        let original_data = b"{\"key\": \"value\"}\n\n";
        encoder.write_all(original_data).unwrap();
        let compressed_data = encoder.finish().unwrap();
        
        let socket = std::io::Cursor::new(compressed_data);
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let (events, _, _) = stats.get_stats().await;
        assert_eq!(events, 1); // Should still be 1 event, second newline ignored
    }

    #[tokio::test]
    async fn test_tcp_absorber_decompression_metrics_with_trailing_space() {
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;
        
        // Prepare gzipped data with trailing space
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        let original_data = b"{\"key\": \"value\"}\n ";
        encoder.write_all(original_data).unwrap();
        let compressed_data = encoder.finish().unwrap();
        
        let socket = std::io::Cursor::new(compressed_data);
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let (events, _, _) = stats.get_stats().await;
        // If this triggers "Failed to validate message", it will be logged.
        // In a test, we can't easily check logs, but we can check if events is 1.
        assert_eq!(events, 1); 
    }

    #[tokio::test]
    async fn test_tcp_absorber_uncompressed_metrics_direct() {
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;
        
        let original_data = b"{\"key\": \"value\"}\n";
        let original_len = original_data.len();
        
        let socket = std::io::Cursor::new(original_data);
        
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let (events, raw_bytes, decomp_bytes) = stats.get_stats().await;
        
        assert_eq!(events, 1);
        assert_eq!(raw_bytes, original_len);
        assert_eq!(decomp_bytes, original_len);
    }
}
