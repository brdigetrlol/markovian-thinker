//! Integration tests for the RustML Sentiment API

use rustml_sentiment_api::{
    SentimentAnalyzer, SentimentClass, TextClassifier, Category,
};

#[test]
fn test_sentiment_analysis_positive() {
    let analyzer = SentimentAnalyzer::new().unwrap();

    let texts = vec![
        "This is absolutely wonderful and amazing!",
        "I love this product, it's fantastic!",
        "Best experience ever! Highly recommended!",
    ];

    for text in texts {
        let result = analyzer.analyze(text).unwrap();
        assert_eq!(
            result.sentiment,
            SentimentClass::Positive,
            "Failed for text: {}",
            text
        );
        assert!(result.compound > 0.0, "Compound score should be positive");
    }
}

#[test]
fn test_sentiment_analysis_negative() {
    let analyzer = SentimentAnalyzer::new().unwrap();

    let texts = vec![
        "This is terrible and awful!",
        "I hate this, it's the worst!",
        "Horrible experience, very disappointed.",
    ];

    for text in texts {
        let result = analyzer.analyze(text).unwrap();
        assert_eq!(
            result.sentiment,
            SentimentClass::Negative,
            "Failed for text: {}",
            text
        );
        assert!(result.compound < 0.0, "Compound score should be negative");
    }
}

#[test]
fn test_sentiment_analysis_neutral() {
    let analyzer = SentimentAnalyzer::new().unwrap();

    let texts = vec![
        "The sky is blue.",
        "It is what it is.",
        "The meeting is scheduled for tomorrow.",
    ];

    for text in texts {
        let result = analyzer.analyze(text).unwrap();
        assert_eq!(
            result.sentiment,
            SentimentClass::Neutral,
            "Failed for text: {}",
            text
        );
    }
}

#[test]
fn test_batch_sentiment_analysis() {
    let analyzer = SentimentAnalyzer::new().unwrap();

    let texts = vec![
        "Great product!".to_string(),
        "Terrible service.".to_string(),
        "The item is blue.".to_string(),
    ];

    let results = analyzer.analyze_batch(&texts).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].sentiment, SentimentClass::Positive);
    assert_eq!(results[1].sentiment, SentimentClass::Negative);
    // Third text should be neutral (factual statement)
    assert!(matches!(results[2].sentiment, SentimentClass::Neutral | SentimentClass::Positive));
}

#[test]
fn test_text_classification_technology() {
    let classifier = TextClassifier::new();

    let texts = vec![
        "Machine learning and AI are transforming the tech industry",
        "The new software algorithm uses neural networks",
        "Cloud computing and blockchain technology",
    ];

    for text in texts {
        let result = classifier.classify(text).unwrap();
        assert_eq!(
            result.category,
            Category::Technology,
            "Failed for text: {}",
            text
        );
    }
}

#[test]
fn test_text_classification_sports() {
    let classifier = TextClassifier::new();

    let text = "The football team won the championship game last night";
    let result = classifier.classify(text).unwrap();
    assert_eq!(result.category, Category::Sports);
}

#[test]
fn test_text_classification_business() {
    let classifier = TextClassifier::new();

    let text = "The company's revenue increased due to strong sales and marketing";
    let result = classifier.classify(text).unwrap();
    assert_eq!(result.category, Category::Business);
}

#[test]
fn test_text_classification_batch() {
    let classifier = TextClassifier::new();

    let texts = vec![
        "AI and machine learning technology".to_string(),
        "Football championship game".to_string(),
        "Business revenue and profit".to_string(),
    ];

    let results = classifier.classify_batch(&texts).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].category, Category::Technology);
    assert_eq!(results[1].category, Category::Sports);
    assert_eq!(results[2].category, Category::Business);
}

#[test]
fn test_sentiment_trend_analysis() {
    let analyzer = SentimentAnalyzer::new().unwrap();

    let texts = vec![
        "Amazing!".to_string(),
        "Wonderful!".to_string(),
        "Great!".to_string(),
        "Fantastic!".to_string(),
        "Excellent!".to_string(),
    ];

    let trend = analyzer.analyze_trend(&texts).unwrap();
    assert_eq!(trend.total_analyzed, 5);
    assert_eq!(trend.positive_count, 5);
    assert_eq!(trend.negative_count, 0);
    assert_eq!(trend.neutral_count, 0);
    assert_eq!(trend.overall_sentiment, SentimentClass::Positive);
}

#[test]
fn test_mixed_sentiment_trend() {
    let analyzer = SentimentAnalyzer::new().unwrap();

    let texts = vec![
        "Great!".to_string(),
        "Terrible!".to_string(),
        "Okay.".to_string(),
        "Amazing!".to_string(),
        "Awful!".to_string(),
    ];

    let trend = analyzer.analyze_trend(&texts).unwrap();
    assert_eq!(trend.total_analyzed, 5);
    assert!(trend.positive_count > 0);
    assert!(trend.negative_count > 0);
}
