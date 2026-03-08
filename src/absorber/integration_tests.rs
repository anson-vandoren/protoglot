#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{Compression, write::GzEncoder};
    use test_log::test;
    use tokio::io::AsyncWriteExt;

    use crate::{
        absorber::{stats_svc::StatsSvc, tcp::handle_tcp_connection},
        config::MessageType,
    };

    #[test(tokio::test)]
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

        let (events, ..) = stats.get_stats().await;
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

        let (events, ..) = stats.get_stats().await;
        // If this triggers "Failed to validate message", it will be logged.
        // In a test, we can't easily check logs, but we can check if events is 1.
        assert_eq!(events, 1);
    }

    #[tokio::test]
    async fn test_tcp_absorber_zstd_metrics_direct() {
        use async_compression::tokio::write::ZstdEncoder;
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;

        let original_data = b"{\"key\": \"value\"}\n";
        let mut encoder = ZstdEncoder::new(Vec::new());
        encoder.write_all(original_data).await.unwrap();
        encoder.shutdown().await.unwrap();
        let compressed_data = encoder.into_inner();

        let compressed_len = compressed_data.len();
        let original_len = original_data.len();

        let socket = std::io::Cursor::new(compressed_data);
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let (events, raw_bytes, decomp_bytes) = stats.get_stats().await;
        assert_eq!(events, 1);
        assert_eq!(raw_bytes, compressed_len);
        assert_eq!(decomp_bytes, original_len);
    }

    #[tokio::test]
    async fn test_tcp_absorber_lz4_metrics_direct() {
        use async_compression::tokio::write::Lz4Encoder;
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;

        let original_data = b"{\"key\": \"value\"}\n";
        let mut encoder = Lz4Encoder::new(Vec::new());
        encoder.write_all(original_data).await.unwrap();
        encoder.shutdown().await.unwrap();
        let compressed_data = encoder.into_inner();

        let compressed_len = compressed_data.len();
        let original_len = original_data.len();

        let socket = std::io::Cursor::new(compressed_data);
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let (events, raw_bytes, decomp_bytes) = stats.get_stats().await;
        assert_eq!(events, 1);
        assert_eq!(raw_bytes, compressed_len);
        assert_eq!(decomp_bytes, original_len);
    }

    #[tokio::test]
    async fn test_tcp_absorber_snappy_metrics_direct() {
        let stats = StatsSvc::run(1000);
        let message_type = MessageType::NdJson;

        let original_data = b"{\"key\": \"value\"}\n";
        let mut encoder = snap::write::FrameEncoder::new(Vec::new());
        encoder.write_all(original_data).unwrap();
        let compressed_data = encoder.into_inner().unwrap();

        let compressed_len = compressed_data.len();
        let original_len = original_data.len();

        let socket = std::io::Cursor::new(compressed_data);
        handle_tcp_connection(socket, &stats, &message_type).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let (events, raw_bytes, decomp_bytes) = stats.get_stats().await;
        assert_eq!(events, 1);
        assert_eq!(raw_bytes, compressed_len);
        assert_eq!(decomp_bytes, original_len);
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

    #[test(tokio::test)]
    async fn test_http_absorber_multi_decompression() {
        use crate::{
            absorber::Absorber,
            config::{ListenAddress, Protocol, absorber::AbsorberConfig},
        };

        let port = 12346;
        let config = AbsorberConfig {
            listen_addresses: vec![ListenAddress {
                host: "127.0.0.1".to_string(),
                port,
                protocol: Protocol::Http,
            }],
            update_interval: 100,
            message_type: MessageType::NdJson,
            ..Default::default()
        };

        let absorber = Absorber::new(config);

        tokio::spawn(async move {
            absorber.run().await.unwrap();
        });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}", port);

        let data = b"{\"foo\":\"bar\"}\n";

        // Test Gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        let body = encoder.finish().unwrap();
        let res = client
            .post(&url)
            .header("Content-Encoding", "gzip")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);

        // Test Zstd
        use async_compression::tokio::write::ZstdEncoder;
        let mut encoder = ZstdEncoder::new(Vec::new());
        encoder.write_all(data).await.unwrap();
        encoder.shutdown().await.unwrap();
        let body = encoder.into_inner();
        let res = client
            .post(&url)
            .header("Content-Encoding", "zstd")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);

        // Test LZ4
        use async_compression::tokio::write::Lz4Encoder;
        let mut encoder = Lz4Encoder::new(Vec::new());
        encoder.write_all(data).await.unwrap();
        encoder.shutdown().await.unwrap();
        let body = encoder.into_inner();
        let res = client.post(&url).header("Content-Encoding", "lz4").body(body).send().await.unwrap();
        assert_eq!(res.status(), 200);

        // Test Brotli
        use async_compression::tokio::write::BrotliEncoder;
        let mut encoder = BrotliEncoder::new(Vec::new());
        encoder.write_all(data).await.unwrap();
        encoder.shutdown().await.unwrap();
        let body = encoder.into_inner();
        let res = client.post(&url).header("Content-Encoding", "br").body(body).send().await.unwrap();
        assert_eq!(res.status(), 200);

        // Test Snappy
        let mut encoder = snap::write::FrameEncoder::new(Vec::new());
        encoder.write_all(data).unwrap();
        let body = encoder.into_inner().unwrap();
        let res = client
            .post(&url)
            .header("Content-Encoding", "snappy")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
}
