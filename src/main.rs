mod simple_plotter;

use simple_plotter::SimplePlotter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 HFT Order Processing Benchmark Analyzer");
    
    let mut plotter = SimplePlotter::new();
    
    // Add sample benchmark results (replace with actual results from your benchmarks)
    plotter.add_sample_results();
    
    // Print visual analysis
    plotter.print_ascii_chart();
    plotter.print_comparison_table();
    plotter.print_scalability_analysis();
    
    // Generate complete report
    plotter.generate_complete_report("hft_benchmark_report")?;
    
    println!("\n✅ Benchmark analysis complete!");
    println!("💡 Run 'cargo bench' first to get real benchmark data");
    
    Ok(())
}
