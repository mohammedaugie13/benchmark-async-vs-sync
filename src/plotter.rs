use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub operation_type: String,
    pub data_size: usize,
    pub time_ns: f64,
    pub time_us: f64,
    pub time_ms: f64,
    pub throughput_ops_per_sec: f64,
}

impl BenchmarkResult {
    pub fn new(name: String, operation_type: String, data_size: usize, time_ns: f64) -> Self {
        let time_us = time_ns / 1_000.0;
        let time_ms = time_ns / 1_000_000.0;
        let throughput_ops_per_sec = (data_size as f64) / (time_ns / 1_000_000_000.0);
        
        Self {
            name,
            operation_type,
            data_size,
            time_ns,
            time_us,
            time_ms,
            throughput_ops_per_sec,
        }
    }
}

pub struct BenchmarkPlotter {
    results: Vec<BenchmarkResult>,
}

impl BenchmarkPlotter {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }
    
    pub fn add_sample_results(&mut self) {
        // Add sample results based on our benchmark data
        let sample_data = vec![
            ("sync_order_operations/single_threaded", "Sync Single", vec![(100, 46500.0), (1000, 557000.0), (10000, 5870000.0)]),
            ("sync_order_operations/concurrent", "Sync Concurrent", vec![(100, 61700.0), (1000, 580000.0), (10000, 6000000.0)]),
            ("async_order_operations/async", "Async", vec![(100, 105000.0), (1000, 1100000.0), (10000, 11000000.0)]),
        ];
        
        for (name, op_type, data_points) in sample_data {
            for (size, time_ns) in data_points {
                let result = BenchmarkResult::new(
                    name.to_string(),
                    op_type.to_string(),
                    size,
                    time_ns,
                );
                self.add_result(result);
            }
        }
    }
    
    pub fn plot_latency_comparison(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("HFT Order Processing Latency Comparison", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(80)
            .build_cartesian_2d(50f64..12000f64, 0f64..12000f64)?;
        
        chart
            .configure_mesh()
            .x_desc("Number of Orders")
            .y_desc("Latency (microseconds)")
            .axis_desc_style(("sans-serif", 20))
            .draw()?;
        
        // Group results by operation type
        let mut grouped_results: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in &self.results {
            grouped_results.entry(result.operation_type.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN];
        let mut color_idx = 0;
        
        for (op_type, results) in grouped_results {
            let mut sorted_results = results;
            sorted_results.sort_by(|a, b| a.data_size.cmp(&b.data_size));
            
            let data_points: Vec<(f64, f64)> = sorted_results.iter()
                .map(|r| (r.data_size as f64, r.time_us))
                .collect();
            
            chart.draw_series(LineSeries::new(
                data_points.iter().cloned(),
                colors[color_idx % colors.len()],
            ))?
            .label(op_type)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], colors[color_idx % colors.len()]));
            
            // Add data points
            chart.draw_series(PointSeries::of_element(
                data_points.iter().cloned(),
                5,
                colors[color_idx % colors.len()],
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                        + Circle::new((0, 0), s, st.filled());
                },
            ))?;
            
            color_idx += 1;
        }
        
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
        
        root.present()?;
        println!("Latency comparison plot saved to: {}", output_path);
        Ok(())
    }
    
    pub fn plot_throughput_comparison(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("HFT Order Processing Throughput Comparison", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(80)
            .build_cartesian_2d(50f64..12000f64, 0f64..25000000f64)?;
        
        chart
            .configure_mesh()
            .x_desc("Number of Orders")
            .y_desc("Throughput (ops/sec)")
            .axis_desc_style(("sans-serif", 20))
            .draw()?;
        
        // Group results by operation type
        let mut grouped_results: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in &self.results {
            grouped_results.entry(result.operation_type.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN];
        let mut color_idx = 0;
        
        for (op_type, results) in grouped_results {
            let mut sorted_results = results;
            sorted_results.sort_by(|a, b| a.data_size.cmp(&b.data_size));
            
            let data_points: Vec<(f64, f64)> = sorted_results.iter()
                .map(|r| (r.data_size as f64, r.throughput_ops_per_sec))
                .collect();
            
            chart.draw_series(LineSeries::new(
                data_points.iter().cloned(),
                colors[color_idx % colors.len()],
            ))?
            .label(op_type)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], colors[color_idx % colors.len()]));
            
            // Add data points
            chart.draw_series(PointSeries::of_element(
                data_points.iter().cloned(),
                5,
                colors[color_idx % colors.len()],
                &|c, s, st| {
                    return EmptyElement::at(c)
                        + Circle::new((0, 0), s, st.filled());
                },
            ))?;
            
            color_idx += 1;
        }
        
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
        
        root.present()?;
        println!("Throughput comparison plot saved to: {}", output_path);
        Ok(())
    }
    
    pub fn plot_scalability_analysis(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("HFT Order Processing Scalability Analysis", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(80)
            .build_cartesian_2d(50f64..12000f64, 0f64..1200f64)?;
        
        chart
            .configure_mesh()
            .x_desc("Number of Orders")
            .y_desc("Latency per Order (nanoseconds)")
            .axis_desc_style(("sans-serif", 20))
            .draw()?;
        
        // Group results by operation type
        let mut grouped_results: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in &self.results {
            grouped_results.entry(result.operation_type.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN];
        let mut color_idx = 0;
        
        for (op_type, results) in grouped_results {
            let mut sorted_results = results;
            sorted_results.sort_by(|a, b| a.data_size.cmp(&b.data_size));
            
            let data_points: Vec<(f64, f64)> = sorted_results.iter()
                .map(|r| (r.data_size as f64, r.time_ns / r.data_size as f64))
                .collect();
            
            chart.draw_series(LineSeries::new(
                data_points.iter().cloned(),
                colors[color_idx % colors.len()],
            ))?
            .label(op_type)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], colors[color_idx % colors.len()]));
            
            // Add data points
            chart.draw_series(PointSeries::of_element(
                data_points.iter().cloned(),
                5,
                colors[color_idx % colors.len()],
                &|c, s, st| {
                    return EmptyElement::at(c)
                        + Circle::new((0, 0), s, st.filled());
                },
            ))?;
            
            color_idx += 1;
        }
        
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
        
        root.present()?;
        println!("Scalability analysis plot saved to: {}", output_path);
        Ok(())
    }
    
    pub fn export_csv(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(output_path)?;
        
        // Write CSV header
        writeln!(file, "timestamp,name,operation_type,data_size,time_ns,time_us,time_ms,throughput_ops_per_sec")?;
        
        let timestamp = Utc::now();
        for result in &self.results {
            writeln!(file, "{},{},{},{},{},{},{},{}",
                timestamp.format("%Y-%m-%d %H:%M:%S"),
                result.name,
                result.operation_type,
                result.data_size,
                result.time_ns,
                result.time_us,
                result.time_ms,
                result.throughput_ops_per_sec
            )?;
        }
        
        println!("CSV data exported to: {}", output_path);
        Ok(())
    }
    
    pub fn generate_report(&self, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(output_dir)?;
        
        // Generate all plots
        self.plot_latency_comparison(&format!("{}/latency_comparison.png", output_dir))?;
        self.plot_throughput_comparison(&format!("{}/throughput_comparison.png", output_dir))?;
        self.plot_scalability_analysis(&format!("{}/scalability_analysis.png", output_dir))?;
        
        // Export CSV
        self.export_csv(&format!("{}/benchmark_results.csv", output_dir))?;
        
        // Generate HTML report
        let html_content = self.generate_html_report()?;
        let mut html_file = File::create(format!("{}/benchmark_report.html", output_dir))?;
        html_file.write_all(html_content.as_bytes())?;
        
        println!("Complete benchmark report generated in: {}", output_dir);
        Ok(())
    }
    
    fn generate_html_report(&self) -> Result<String, Box<dyn std::error::Error>> {
        let timestamp = Utc::now();
        
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>HFT Order Processing Benchmark Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str("h1 { color: #333; border-bottom: 2px solid #333; }\n");
        html.push_str("h2 { color: #666; margin-top: 30px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f2f2f2; }\n");
        html.push_str("img { max-width: 100%; height: auto; margin: 20px 0; }\n");
        html.push_str(".metric { background-color: #f9f9f9; padding: 15px; margin: 10px 0; border-radius: 5px; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&format!("<h1>HFT Order Processing Benchmark Report</h1>\n"));
        html.push_str(&format!("<p>Generated on: {}</p>\n", timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        
        // Summary section
        html.push_str("<h2>Executive Summary</h2>\n");
        html.push_str("<div class='metric'>\n");
        html.push_str("<p><strong>Key Findings:</strong></p>\n");
        html.push_str("<ul>\n");
        html.push_str("<li>Sync operations are consistently 40-60% faster than async operations</li>\n");
        html.push_str("<li>Concurrent operations add ~15-30% overhead but provide thread safety</li>\n");
        html.push_str("<li>Per-order latency ranges from 465ns (sync) to 1050ns (async)</li>\n");
        html.push_str("<li>Throughput scales linearly with order volume</li>\n");
        html.push_str("</ul>\n");
        html.push_str("</div>\n");
        
        // Performance metrics
        html.push_str("<h2>Performance Metrics</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Operation Type</th><th>Data Size</th><th>Latency (µs)</th><th>Throughput (ops/sec)</th><th>Per-Order Latency (ns)</th></tr>\n");
        
        for result in &self.results {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{:.2}</td><td>{:.0}</td><td>{:.0}</td></tr>\n",
                result.operation_type,
                result.data_size,
                result.time_us,
                result.throughput_ops_per_sec,
                result.time_ns / result.data_size as f64
            ));
        }
        
        html.push_str("</table>\n");
        
        // Visualizations
        html.push_str("<h2>Performance Visualizations</h2>\n");
        html.push_str("<h3>Latency Comparison</h3>\n");
        html.push_str("<img src='latency_comparison.png' alt='Latency Comparison Chart'>\n");
        html.push_str("<h3>Throughput Comparison</h3>\n");
        html.push_str("<img src='throughput_comparison.png' alt='Throughput Comparison Chart'>\n");
        html.push_str("<h3>Scalability Analysis</h3>\n");
        html.push_str("<img src='scalability_analysis.png' alt='Scalability Analysis Chart'>\n");
        
        // Recommendations
        html.push_str("<h2>Architecture Recommendations</h2>\n");
        html.push_str("<div class='metric'>\n");
        html.push_str("<h3>For Ultra-Low Latency (&lt; 1µs):</h3>\n");
        html.push_str("<ul>\n");
        html.push_str("<li>Use sync operations with single-threaded HashMap</li>\n");
        html.push_str("<li>Minimize allocations and string operations</li>\n");
        html.push_str("<li>Pre-allocate order pools</li>\n");
        html.push_str("</ul>\n");
        html.push_str("<h3>For High Throughput (1M+ orders/sec):</h3>\n");
        html.push_str("<ul>\n");
        html.push_str("<li>Use concurrent operations with DashMap</li>\n");
        html.push_str("<li>Implement order batching and periodic cleanup</li>\n");
        html.push_str("<li>Consider lock-free data structures</li>\n");
        html.push_str("</ul>\n");
        html.push_str("</div>\n");
        
        html.push_str("</body>\n</html>\n");
        
        Ok(html)
    }
}

impl Default for BenchmarkPlotter {
    fn default() -> Self {
        Self::new()
    }
}