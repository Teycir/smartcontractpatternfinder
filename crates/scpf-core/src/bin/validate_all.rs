use std::fs;
use std::path::Path;
use scpf_core::PatternValidator;

fn main() -> anyhow::Result<()> {
    let validator = PatternValidator::new();
    let template_dir = Path::new("templates");
    
    let mut total_templates = 0;
    let mut total_patterns = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;
    
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          SCPF Pattern Validation Report                  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let mut entries: Vec<_> = fs::read_dir(template_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "yaml").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.path());
    
    for entry in entries {
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        
        match validator.validate_template(&content) {
            Ok(validation) => {
                total_templates += 1;
                total_patterns += validation.total;
                total_passed += validation.passed;
                total_failed += validation.failed;
                
                if validation.failed > 0 {
                    validation.print_report();
                } else if validation.total > 0 {
                    println!("✅ {} - All {} patterns valid", path.display(), validation.total);
                }
            }
            Err(e) => {
                println!("⚠️  {} - Parse error: {}", path.display(), e);
            }
        }
    }
    
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY                              ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Templates scanned:  {:>4}                                ║", total_templates);
    println!("║  Total patterns:     {:>4}                                ║", total_patterns);
    println!("║  Passed:             {:>4} ✅                             ║", total_passed);
    println!("║  Failed:             {:>4} ❌                             ║", total_failed);
    println!("║  Success Rate:       {:>5.1}%                             ║", 
        if total_patterns > 0 { (total_passed as f64 / total_patterns as f64) * 100.0 } else { 0.0 });
    println!("╚══════════════════════════════════════════════════════════╝");
    
    if total_failed > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}
