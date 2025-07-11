use std::collections::HashMap;
use std::fs;
use std::path::Path;
use benchmark_async_vs_sync::simple_plotter::{SimplePlotter, BenchmarkResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 HFT Benchmark Results Plotter");
    
    let mut plotter = SimplePlotter::new();
    
    // Try to parse criterion results from target/criterion directory
    let criterion_dir = "target/criterion";
    if Path::new(criterion_dir).exists() {
        println!("📁 Found criterion results directory, parsing...");
        if let Ok(results) = parse_criterion_results(criterion_dir) {
            for result in results {
                plotter.add_result(result);
            }
            println!("✅ Parsed {} benchmark results from criterion data", plotter.results.len());
        } else {
            println!("⚠️  Could not parse criterion results, using sample data");
            plotter.add_sample_results();
        }
    } else {
        println!("⚠️  No criterion results found, using sample data");
        println!("💡 Run 'cargo bench' first to generate real benchmark data");
        plotter.add_sample_results();
    }
    
    // Generate the complete report
    let output_dir = "hft_benchmark_report";
    plotter.generate_complete_report(output_dir)?;
    
    println!("\n🎉 Benchmark visualization complete!");
    println!("📊 Open {}/benchmark_report.html to view the interactive report", output_dir);
    println!("📈 Charts saved as PNG files in {} directory", output_dir);
    println!("📄 Raw data exported to {}/benchmark_results.csv", output_dir);
    
    // Print quick summary
    print_performance_summary(&plotter)?;
    
    Ok(())
}

fn parse_criterion_results(criterion_dir: &str) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    // Walk through criterion directory structure
    for entry in fs::read_dir(criterion_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            
            // Look for estimates.json files in each benchmark directory
            let estimates_path = path.join("base").join("estimates.json");
            if estimates_path.exists() {
                if let Ok(estimate_data) = fs::read_to_string(&estimates_path) {
                    if let Ok(estimate_json) = serde_json::from_str::<serde_json::Value>(&estimate_data) {
                        if let Some(mean_estimate) = estimate_json.get("mean") {
                            if let Some(point_estimate) = mean_estimate.get("point_estimate") {
                                if let Some(time_ns) = point_estimate.as_f64() {
                                    // Parse benchmark name and extract info
                                    let (operation_type, data_size) = parse_benchmark_name(&dir_name);
                                    
                                    let result = BenchmarkResult::new(
                                        dir_name.to_string(),
                                        operation_type,
                                        data_size,
                                        time_ns,
                                    );
                                    results.push(result);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(results)
}

fn parse_benchmark_name(name: &str) -> (String, usize) {
    // Parse benchmark names like "sync_order_operations/single_threaded/100"
    let parts: Vec<&str> = name.split('/').collect();
    
    let operation_type = if parts.len() >= 2 {
        match (parts[0], parts[1]) {
            ("sync_order_operations", "single_threaded") => "Sync Single".to_string(),
            ("sync_order_operations", "concurrent") => "Sync Concurrent".to_string(),
            ("async_order_operations", "async") => "Async".to_string(),
            ("order_flattening", "sync") => "Flatten Sync".to_string(),
            ("order_flattening", "async") => "Flatten Async".to_string(),
            ("hft_order_simulation", _) => "HFT Simulation".to_string(),
            ("trillion_scale_orders", _) => "Trillion Scale".to_string(),
            _ => format!("{} {}", parts[0], parts.get(1).unwrap_or(&"")),
        }
    } else {
        parts[0].to_string()
    };
    
    let data_size = if let Some(last_part) = parts.last() {
        last_part.parse::<usize>().unwrap_or(1)
    } else {
        1
    };
    
    (operation_type, data_size)
}

fn print_performance_summary(plotter: &SimplePlotter) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 PERFORMANCE SUMMARY");
    println!("{}", "=".repeat(50));
    
    // Group by operation type
    let mut grouped: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
    for result in &plotter.results {
        grouped.entry(result.operation_type.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }
    
    for (op_type, mut results) in grouped {
        results.sort_by(|a, b| a.data_size.cmp(&b.data_size));
        
        println!("\n🔸 {}", op_type);
        for result in results {
            let per_order_ns = result.time_ns / result.data_size as f64;
            println!("  {} orders: {:.1}µs ({:.0}ns/order, {:.0}M ops/sec)", 
                result.data_size,
                result.time_us,
                per_order_ns,
                result.throughput_ops_per_sec / 1_000_000.0
            );
        }
    }
    
    println!("\n💡 TIP: Lower latency and higher throughput = better performance");
    println!("🎯 For HFT: Target < 1µs total latency and < 500ns per order");
    
    Ok(())
}