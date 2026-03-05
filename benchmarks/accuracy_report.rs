use anyhow::Result;
use scpf_core::{Scanner, TemplateLoader};
use std::path::PathBuf;

mod accuracy;
use accuracy::AccuracyEvaluator;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== SCPF Accuracy Evaluation ===\n");

    let ground_truth_path = PathBuf::from("benchmarks/ground-truth.json");
    let evaluator = AccuracyEvaluator::load(&ground_truth_path)?;

    let templates_dir = PathBuf::from("templates");
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    println!("Loaded {} templates", templates.len());
    println!("Scanning benchmark contracts...\n");

    let scanner = Scanner::new(templates)?;
    let benchmark_dir = PathBuf::from("benchmarks");

    let mut all_findings = Vec::new();

    for entry in walkdir::WalkDir::new(&benchmark_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sol"))
    {
        let path = entry.path();
        if let Ok(content) = std::fs::read_to_string(path) {
            let findings = scanner.scan(&content, path.to_path_buf())?;
            all_findings.extend(findings);
        }
    }

    println!("Found {} total findings\n", all_findings.len());

    let metrics = evaluator.evaluate(&all_findings);
    evaluator.print_report(&metrics);

    if metrics.f1_score < 0.80 {
        eprintln!("\n⚠️  WARNING: F1 score below 80% threshold");
    }

    Ok(())
}
