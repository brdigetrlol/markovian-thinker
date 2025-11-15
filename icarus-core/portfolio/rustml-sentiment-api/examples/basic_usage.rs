//! Basic usage example for the RustML Sentiment API library

use rustml_sentiment_api::{SentimentAnalyzer, TextClassifier};

fn main() -> anyhow::Result<()> {
    println!("🦀 RustML Sentiment API - Basic Usage Example\n");

    // Initialize analyzers
    let sentiment_analyzer = SentimentAnalyzer::new()?;
    let text_classifier = TextClassifier::new();

    // Example 1: Sentiment Analysis
    println!("📊 Example 1: Sentiment Analysis");
    println!("─────────────────────────────────");

    let texts = vec![
        "This product is absolutely amazing! Best purchase ever!",
        "Terrible experience. Very disappointed with the service.",
        "The item arrived on time. It works as expected.",
    ];

    for text in &texts {
        let result = sentiment_analyzer.analyze(text)?;
        println!("\nText: \"{}\"", text);
        println!("Sentiment: {:?}", result.sentiment);
        println!("Compound Score: {:.3}", result.compound);
        println!("Positive: {:.3}, Negative: {:.3}, Neutral: {:.3}",
                 result.positive, result.negative, result.neutral);
        println!("Confidence: {:.1}%", result.confidence * 100.0);
    }

    // Example 2: Text Classification
    println!("\n\n📁 Example 2: Text Classification");
    println!("─────────────────────────────────");

    let classification_texts = vec![
        "Machine learning and artificial intelligence are revolutionizing technology",
        "The football team secured victory in the championship finals",
        "Stock market sees significant gains as investors show confidence",
    ];

    for text in &classification_texts {
        let result = text_classifier.classify(text)?;
        println!("\nText: \"{}\"", text);
        println!("Category: {:?}", result.category);
        println!("Confidence: {:.1}%", result.confidence * 100.0);
        println!("Keywords detected: {:?}", result.keywords);
    }

    // Example 3: Batch Processing
    println!("\n\n🔄 Example 3: Batch Processing");
    println!("─────────────────────────────────");

    let batch_texts = vec![
        "Excellent service and quality!".to_string(),
        "Not satisfied with the product.".to_string(),
        "Average experience overall.".to_string(),
    ];

    let batch_results = sentiment_analyzer.analyze_batch(&batch_texts)?;
    println!("\nProcessed {} texts in batch:", batch_results.len());
    for (i, result) in batch_results.iter().enumerate() {
        println!("  {}. {:?} (score: {:.3})", i + 1, result.sentiment, result.compound);
    }

    // Example 4: Trend Analysis
    println!("\n\n📈 Example 4: Trend Analysis");
    println!("─────────────────────────────────");

    let trend_texts = vec![
        "Great product!".to_string(),
        "Love it!".to_string(),
        "Amazing quality!".to_string(),
        "Not what I expected.".to_string(),
        "Fantastic!".to_string(),
    ];

    let trend = sentiment_analyzer.analyze_trend(&trend_texts)?;
    println!("\nAnalyzed {} texts", trend.total_analyzed);
    println!("Positive: {}, Negative: {}, Neutral: {}",
             trend.positive_count, trend.negative_count, trend.neutral_count);
    println!("Average Sentiment: {:.3}", trend.average_compound);
    println!("Overall Trend: {:?}", trend.overall_sentiment);

    println!("\n✅ All examples completed successfully!");

    Ok(())
}
