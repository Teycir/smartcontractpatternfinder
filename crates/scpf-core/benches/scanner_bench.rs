use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use scpf_core::Scanner;
use scpf_types::{Pattern, Severity, Template};
use std::path::PathBuf;

fn create_test_template() -> Template {
    Template {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test template".to_string(),
        severity: Severity::High,
        tags: vec!["test".to_string()],
        patterns: vec![
            Pattern {
                id: "call".to_string(),
                pattern: r"\.call\{value:".to_string(),
                message: "External call".to_string(),
            },
            Pattern {
                id: "delegatecall".to_string(),
                pattern: r"\.delegatecall\(".to_string(),
                message: "Delegatecall".to_string(),
            },
            Pattern {
                id: "selfdestruct".to_string(),
                pattern: r"selfdestruct\(".to_string(),
                message: "Selfdestruct".to_string(),
            },
        ],
    }
}

fn generate_contract(lines: usize) -> String {
    let mut contract = String::from("pragma solidity ^0.8.0;\n\ncontract Test {\n");
    for i in 0..lines {
        if i % 10 == 0 {
            contract.push_str("    address.call{value: 1 ether}(\"\");\n");
        } else if i % 15 == 0 {
            contract.push_str("    address.delegatecall(\"\");\n");
        } else {
            contract.push_str("    uint256 x = 42;\n");
        }
    }
    contract.push_str("}\n");
    contract
}

fn bench_scan_small(c: &mut Criterion) {
    let scanner = Scanner::new(vec![create_test_template()]).unwrap();
    let source = generate_contract(50);
    
    c.bench_function("scan_small_50_lines", |b| {
        b.iter(|| {
            scanner.scan(black_box(&source), PathBuf::from("test.sol")).unwrap()
        })
    });
}

fn bench_scan_medium(c: &mut Criterion) {
    let scanner = Scanner::new(vec![create_test_template()]).unwrap();
    let source = generate_contract(200);
    
    c.bench_function("scan_medium_200_lines", |b| {
        b.iter(|| {
            scanner.scan(black_box(&source), PathBuf::from("test.sol")).unwrap()
        })
    });
}

fn bench_scan_large(c: &mut Criterion) {
    let scanner = Scanner::new(vec![create_test_template()]).unwrap();
    let source = generate_contract(1000);
    
    c.bench_function("scan_large_1000_lines", |b| {
        b.iter(|| {
            scanner.scan(black_box(&source), PathBuf::from("test.sol")).unwrap()
        })
    });
}

fn bench_scan_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_scaling");
    let scanner = Scanner::new(vec![create_test_template()]).unwrap();
    
    for size in [100, 500, 1000, 2000].iter() {
        let source = generate_contract(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                scanner.scan(black_box(&source), PathBuf::from("test.sol")).unwrap()
            })
        });
    }
    group.finish();
}

fn bench_line_index(c: &mut Criterion) {
    let source = generate_contract(1000);
    
    // Build newlines vector (same as Scanner does internally)
    let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();
    
    // Generate random byte positions to look up
    let positions: Vec<usize> = (0..100)
        .map(|i| (i * source.len()) / 100)
        .collect();
    
    c.bench_function("line_index_lookup", |b| {
        b.iter(|| {
            // Benchmark the line-number lookup operation using partition_point
            for &pos in black_box(&positions) {
                let line_number = newlines.partition_point(|&nl_pos| nl_pos < pos) + 1;
                // Use the result to prevent optimization
                black_box(line_number);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_scan_small,
    bench_scan_medium,
    bench_scan_large,
    bench_scan_scaling,
    bench_line_index
);
criterion_main!(benches);
