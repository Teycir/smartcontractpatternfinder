use anyhow::Result;
use scpf_core::{Scanner, TemplateLoader};
use std::path::PathBuf;

mod accuracy;
use accuracy::{AccuracyEvaluator, AccuracyMetrics};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== SCPF Accuracy Evaluation ===\n");
    
    let ground_truth_path = PathBuf::from("benchmarks/ground-truth.json");
    let evaluator = AccuracyEvaluator::load(&ground_truth_path)?;
    
    let templates_dir = PathBuf::from("templates");
    let loader = TemplateLoader::new(templates_dir);
    let templates = loader.load_all()?;
    
    println!("Loaded {} templates", templates.len());
    println!("Scanning benchmark contracts...\n");
    
    let scanner = Scanner::new(templates);
    let benchmark_dir = PathBuf::from("benchmarks");
    
    let mut all_findings = Vec::new();
    
    for entry in walkdir::WalkDir::new(&benchmark_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sol"))
    {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(path) {
            let findings = scanner.scan_content(&content, path)?;
            all_findings.extend(findings);
        }
    }
    
    println!("Found {} total findings\n", all_findings.len());
    
    let metrics = evaluator.evaluate(&all_findings);
    evaluator.print_report(&metrics);
    
    if metrics.f1_score < 0.80 {
        eprintln!("\n⚠️  WARNING: F1 score below 80% threshold");
        std::process::exit(1);
    }
    
    Ok(())
}
