// Streams
// Continuous data processing (vs. turn-based prompts)

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Stream item types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamItem {
    /// Text data
    Text(String),

    /// Code data
    Code {
        language: String,
        content: String,
    },

    /// Binary data
    Binary(Vec<u8>),

    /// Structured data
    Structured(serde_json::Value),

    /// End of stream
    EndOfStream,
}

/// Stream metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub id: Uuid,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
}

/// Stream message (item + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub metadata: StreamMetadata,
    pub item: StreamItem,
}

impl StreamMessage {
    pub fn new(source: String, item: StreamItem, sequence: u64) -> Self {
        Self {
            metadata: StreamMetadata {
                id: Uuid::new_v4(),
                source,
                timestamp: Utc::now(),
                sequence,
            },
            item,
        }
    }
}

/// Stream processor - Processes continuous streams of data
pub struct StreamProcessor {
    source: String,
    receiver: mpsc::Receiver<StreamMessage>,
    sequence: u64,
}

impl StreamProcessor {
    pub fn new(source: String, receiver: mpsc::Receiver<StreamMessage>) -> Self {
        Self {
            source,
            receiver,
            sequence: 0,
        }
    }

    /// Process next item from stream
    pub async fn next(&mut self) -> Option<StreamMessage> {
        self.receiver.recv().await
    }

    /// Process stream in a loop
    pub async fn process_loop<F>(&mut self, mut handler: F)
    where
        F: FnMut(StreamMessage) -> anyhow::Result<()>,
    {
        while let Some(message) = self.next().await {
            // Check for end of stream
            if matches!(message.item, StreamItem::EndOfStream) {
                tracing::info!("End of stream from {}", self.source);
                break;
            }

            // Process item
            if let Err(e) = handler(message) {
                tracing::error!("Stream processing error: {}", e);
            }

            self.sequence += 1;
        }
    }
}

/// Stream producer - Sends data to stream
pub struct StreamProducer {
    source: String,
    sender: mpsc::Sender<StreamMessage>,
    sequence: u64,
}

impl StreamProducer {
    pub fn new(source: String, sender: mpsc::Sender<StreamMessage>) -> Self {
        Self {
            source,
            sender,
            sequence: 0,
        }
    }

    /// Send item to stream
    pub async fn send(&mut self, item: StreamItem) -> anyhow::Result<()> {
        let message = StreamMessage::new(self.source.clone(), item, self.sequence);
        self.sender.send(message).await?;
        self.sequence += 1;
        Ok(())
    }

    /// Signal end of stream
    pub async fn end(&mut self) -> anyhow::Result<()> {
        self.send(StreamItem::EndOfStream).await
    }
}

/// Create a stream channel
pub fn create_stream(source: String, buffer_size: usize) -> (StreamProducer, StreamProcessor) {
    let (sender, receiver) = mpsc::channel(buffer_size);
    let producer = StreamProducer::new(source.clone(), sender);
    let processor = StreamProcessor::new(source, receiver);
    (producer, processor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_basic() {
        let (mut producer, mut processor) = create_stream("test".to_string(), 10);

        // Send some items
        producer.send(StreamItem::Text("Hello".to_string())).await.unwrap();
        producer.send(StreamItem::Text("World".to_string())).await.unwrap();
        producer.end().await.unwrap();

        // Receive items
        let msg1 = processor.next().await.unwrap();
        assert!(matches!(msg1.item, StreamItem::Text(_)));

        let msg2 = processor.next().await.unwrap();
        assert!(matches!(msg2.item, StreamItem::Text(_)));

        let msg3 = processor.next().await.unwrap();
        assert!(matches!(msg3.item, StreamItem::EndOfStream));
    }

    #[tokio::test]
    async fn test_stream_process_loop() {
        let (mut producer, mut processor) = create_stream("test".to_string(), 10);

        // Send items in background
        tokio::spawn(async move {
            for i in 0..5 {
                producer.send(StreamItem::Text(format!("Item {}", i))).await.unwrap();
            }
            producer.end().await.unwrap();
        });

        // Process with loop
        let mut count = 0;
        processor.process_loop(|_msg| {
            count += 1;
            Ok(())
        }).await;

        assert_eq!(count, 5);
    }
}
