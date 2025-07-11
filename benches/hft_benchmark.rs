use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dashmap::DashMap;
use futures::future::join_all;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    order_id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: u64,
    price: f64,
    filled_quantity: u64,
    remaining_quantity: u64,
    status: OrderStatus,
    timestamp: u64,
    client_id: String,
}

impl Order {
    fn new(order_id: u64, symbol: String, client_id: String) -> Self {
        let mut rng = thread_rng();
        let quantity = rng.gen_range(1..10000);
        Self {
            order_id,
            symbol,
            side: if rng.gen_bool(0.5) { OrderSide::Buy } else { OrderSide::Sell },
            order_type: match rng.gen_range(0..4) {
                0 => OrderType::Market,
                1 => OrderType::Limit,
                2 => OrderType::Stop,
                _ => OrderType::StopLimit,
            },
            quantity,
            price: rng.gen_range(10.0..1000.0),
            filled_quantity: 0,
            remaining_quantity: quantity,
            status: OrderStatus::New,
            timestamp: rng.gen_range(1600000000..1700000000),
            client_id,
        }
    }
    
    fn update_fill(&mut self, fill_quantity: u64) {
        let actual_fill = std::cmp::min(fill_quantity, self.remaining_quantity);
        self.filled_quantity += actual_fill;
        self.remaining_quantity -= actual_fill;
        
        if self.remaining_quantity == 0 {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}

// Sync operations
fn sync_order_operations(data_size: usize) {
    let mut order_map: HashMap<String, Order> = HashMap::new();
    let mut rng = thread_rng();
    
    // Write operations - Create orders
    for i in 0..data_size {
        let order_id = format!("ORD_{}", i);
        let symbol = format!("STOCK_{}", i % 1000); // Cycle through 1000 symbols
        let client_id = format!("CLIENT_{}", i % 100); // 100 clients
        let order = Order::new(i as u64, symbol, client_id);
        order_map.insert(order_id, order);
    }
    
    // Read operations - Query orders
    for i in 0..data_size {
        let order_id = format!("ORD_{}", i);
        black_box(order_map.get(&order_id));
    }
    
    // Update operations - Fill orders
    for i in 0..data_size {
        let order_id = format!("ORD_{}", i);
        if let Some(order) = order_map.get_mut(&order_id) {
            if order.remaining_quantity > 0 {
                let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                order.update_fill(fill_qty);
            }
        }
    }
}

fn sync_concurrent_order_operations(data_size: usize) {
    let order_map: Arc<DashMap<String, Order>> = Arc::new(DashMap::new());
    let mut rng = thread_rng();
    
    // Write operations - Create orders
    for i in 0..data_size {
        let order_id = format!("ORD_{}", i);
        let symbol = format!("STOCK_{}", i % 1000);
        let client_id = format!("CLIENT_{}", i % 100);
        let order = Order::new(i as u64, symbol, client_id);
        order_map.insert(order_id, order);
    }
    
    // Read operations - Query orders
    for i in 0..data_size {
        let order_id = format!("ORD_{}", i);
        black_box(order_map.get(&order_id));
    }
    
    // Update operations - Fill orders
    for i in 0..data_size {
        let order_id = format!("ORD_{}", i);
        if let Some(mut order) = order_map.get_mut(&order_id) {
            if order.remaining_quantity > 0 {
                let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                order.update_fill(fill_qty);
            }
        }
    }
}

// Async operations
async fn async_order_operations(data_size: usize) {
    let order_map: Arc<DashMap<String, Order>> = Arc::new(DashMap::new());
    
    // Write operations - Create orders
    let write_tasks: Vec<_> = (0..data_size)
        .map(|i| {
            let order_map = order_map.clone();
            async move {
                let order_id = format!("ORD_{}", i);
                let symbol = format!("STOCK_{}", i % 1000);
                let client_id = format!("CLIENT_{}", i % 100);
                let order = Order::new(i as u64, symbol, client_id);
                order_map.insert(order_id, order);
            }
        })
        .collect();
    
    join_all(write_tasks).await;
    
    // Read operations - Query orders
    let read_tasks: Vec<_> = (0..data_size)
        .map(|i| {
            let order_map = order_map.clone();
            async move {
                let order_id = format!("ORD_{}", i);
                black_box(order_map.get(&order_id));
            }
        })
        .collect();
    
    join_all(read_tasks).await;
    
    // Update operations - Fill orders
    let update_tasks: Vec<_> = (0..data_size)
        .map(|i| {
            let order_map = order_map.clone();
            async move {
                let order_id = format!("ORD_{}", i);
                if let Some(mut order) = order_map.get_mut(&order_id) {
                    if order.remaining_quantity > 0 {
                        let mut rng = thread_rng();
                        let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                        order.update_fill(fill_qty);
                    }
                }
            }
        })
        .collect();
    
    join_all(update_tasks).await;
}

// Order book flattening operations
fn sync_order_flatten(nested_orders: &HashMap<String, HashMap<String, Order>>) -> HashMap<String, Order> {
    let mut flattened = HashMap::new();
    
    for (exchange, orders) in nested_orders {
        for (order_id, order) in orders {
            let flat_key = format!("{}:{}", exchange, order_id);
            flattened.insert(flat_key, order.clone());
        }
    }
    
    flattened
}

async fn async_order_flatten(nested_orders: &HashMap<String, HashMap<String, Order>>) -> HashMap<String, Order> {
    let tasks: Vec<_> = nested_orders
        .iter()
        .map(|(exchange, orders)| {
            let exchange = exchange.clone();
            let orders = orders.clone();
            async move {
                let mut local_flattened = HashMap::new();
                for (order_id, order) in orders {
                    let flat_key = format!("{}:{}", exchange, order_id);
                    local_flattened.insert(flat_key, order);
                }
                local_flattened
            }
        })
        .collect();
    
    let results = join_all(tasks).await;
    let mut flattened = HashMap::new();
    
    for result in results {
        flattened.extend(result);
    }
    
    flattened
}

fn create_nested_order_data(exchanges: usize, orders_per_exchange: usize) -> HashMap<String, HashMap<String, Order>> {
    let mut nested_orders = HashMap::new();
    
    for i in 0..exchanges {
        let exchange = format!("EXCHANGE_{}", i);
        let mut orders = HashMap::new();
        
        for j in 0..orders_per_exchange {
            let order_id = format!("ORD_{}_{}", i, j);
            let symbol = format!("STOCK_{}", j % 100);
            let client_id = format!("CLIENT_{}", j % 50);
            let order = Order::new((i * orders_per_exchange + j) as u64, symbol, client_id);
            orders.insert(order_id, order);
        }
        
        nested_orders.insert(exchange, orders);
    }
    
    nested_orders
}

// Trillion-scale transaction simulation
fn sync_trillion_transactions(batch_size: usize, batches: usize) {
    let order_map: Arc<DashMap<String, Order>> = Arc::new(DashMap::new());
    let mut rng = thread_rng();
    
    for batch in 0..batches {
        for i in 0..batch_size {
            let order_id = format!("ORD_{}_{}", batch, i);
            let symbol = format!("STOCK_{}", i % 1000);
            let client_id = format!("CLIENT_{}", i % 100);
            let order = Order::new((batch * batch_size + i) as u64, symbol, client_id);
            order_map.insert(order_id.clone(), order);
            
            // Simulate order processing
            if let Some(mut order) = order_map.get_mut(&order_id) {
                if order.remaining_quantity > 0 {
                    let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                    order.update_fill(fill_qty);
                }
            }
        }
        
        // Cleanup filled orders periodically
        if batch % 1000 == 0 {
            order_map.retain(|_, order| order.status != OrderStatus::Filled);
        }
    }
}

async fn async_trillion_transactions(batch_size: usize, batches: usize) {
    let order_map: Arc<DashMap<String, Order>> = Arc::new(DashMap::new());
    
    for batch in 0..batches {
        let batch_tasks: Vec<_> = (0..batch_size)
            .map(|i| {
                let order_map = order_map.clone();
                async move {
                    let order_id = format!("ORD_{}_{}", batch, i);
                    let symbol = format!("STOCK_{}", i % 1000);
                    let client_id = format!("CLIENT_{}", i % 100);
                    let order = Order::new((batch * batch_size + i) as u64, symbol, client_id);
                    order_map.insert(order_id.clone(), order);
                    
                    // Simulate order processing
                    if let Some(mut order) = order_map.get_mut(&order_id) {
                        if order.remaining_quantity > 0 {
                            let mut rng = thread_rng();
                            let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                            order.update_fill(fill_qty);
                        }
                    }
                }
            })
            .collect();
        
        join_all(batch_tasks).await;
        
        // Cleanup filled orders periodically
        if batch % 1000 == 0 {
            order_map.retain(|_, order| order.status != OrderStatus::Filled);
        }
    }
}

fn bench_sync_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_order_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("single_threaded", size),
            size,
            |b, &size| {
                b.iter(|| sync_order_operations(size));
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("concurrent", size),
            size,
            |b, &size| {
                b.iter(|| sync_concurrent_order_operations(size));
            },
        );
    }
    
    group.finish();
}

fn bench_async_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_order_operations");
    let rt = Runtime::new().unwrap();
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("async", size),
            size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async_order_operations(size));
                });
            },
        );
    }
    
    group.finish();
}

fn bench_order_flattening(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_flattening");
    let rt = Runtime::new().unwrap();
    
    for exchanges in [5, 10, 20].iter() {
        for orders in [100, 500, 1000].iter() {
            let nested_orders = create_nested_order_data(*exchanges, *orders);
            
            group.bench_with_input(
                BenchmarkId::new("sync", format!("{}x{}", exchanges, orders)),
                &nested_orders,
                |b, data| {
                    b.iter(|| sync_order_flatten(data));
                },
            );
            
            group.bench_with_input(
                BenchmarkId::new("async", format!("{}x{}", exchanges, orders)),
                &nested_orders,
                |b, data| {
                    b.iter(|| {
                        rt.block_on(async_order_flatten(data));
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn bench_hft_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hft_order_simulation");
    let rt = Runtime::new().unwrap();
    
    // Simulate high-frequency trading order scenarios
    let order_updates = 10000;
    
    group.bench_function("sync_hft_orders", |b| {
        b.iter(|| {
            let order_map: Arc<DashMap<String, Order>> = Arc::new(DashMap::new());
            let mut rng = thread_rng();
            
            // Initial order population
            for i in 0..1000 {
                let order_id = format!("ORD_{}", i);
                let symbol = format!("STOCK_{}", i % 100);
                let client_id = format!("CLIENT_{}", i % 50);
                let order = Order::new(i as u64, symbol, client_id);
                order_map.insert(order_id, order);
            }
            
            // High frequency order updates
            for _ in 0..order_updates {
                let order_id = format!("ORD_{}", rng.gen_range(0..1000));
                if let Some(mut order) = order_map.get_mut(&order_id) {
                    if order.remaining_quantity > 0 {
                        let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                        order.update_fill(fill_qty);
                    }
                }
            }
        });
    });
    
    group.bench_function("async_hft_orders", |b| {
        b.iter(|| {
            rt.block_on(async {
                let order_map: Arc<DashMap<String, Order>> = Arc::new(DashMap::new());
                
                // Initial order population
                let init_tasks: Vec<_> = (0..1000)
                    .map(|i| {
                        let order_map = order_map.clone();
                        async move {
                            let order_id = format!("ORD_{}", i);
                            let symbol = format!("STOCK_{}", i % 100);
                            let client_id = format!("CLIENT_{}", i % 50);
                            let order = Order::new(i as u64, symbol, client_id);
                            order_map.insert(order_id, order);
                        }
                    })
                    .collect();
                
                join_all(init_tasks).await;
                
                // High frequency order updates
                let update_tasks: Vec<_> = (0..order_updates)
                    .map(|_| {
                        let order_map = order_map.clone();
                        async move {
                            let mut rng = thread_rng();
                            let order_id = format!("ORD_{}", rng.gen_range(0..1000));
                            if let Some(mut order) = order_map.get_mut(&order_id) {
                                if order.remaining_quantity > 0 {
                                    let fill_qty = rng.gen_range(1..=order.remaining_quantity);
                                    order.update_fill(fill_qty);
                                }
                            }
                        }
                    })
                    .collect();
                
                join_all(update_tasks).await;
            });
        });
    });
    
    group.finish();
}

fn bench_trillion_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("trillion_scale_orders");
    group.sample_size(10); // Reduce sample size for large benchmarks
    group.measurement_time(std::time::Duration::from_secs(60)); // Longer measurement time
    
    let rt = Runtime::new().unwrap();
    
    // Test with different batch sizes to simulate trillion transactions
    // 1 trillion = 1,000,000,000,000
    // We'll use smaller representative samples
    let test_configs = [
        (100_000, 1000),    // 100M transactions
        (1_000_000, 100),   // 100M transactions
        (10_000_000, 10),   // 100M transactions
    ];
    
    for (batch_size, batches) in test_configs.iter() {
        group.bench_with_input(
            BenchmarkId::new("sync_trillion", format!("{}x{}", batch_size, batches)),
            &(*batch_size, *batches),
            |b, &(batch_size, batches)| {
                b.iter(|| sync_trillion_transactions(batch_size, batches));
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("async_trillion", format!("{}x{}", batch_size, batches)),
            &(*batch_size, *batches),
            |b, &(batch_size, batches)| {
                b.iter(|| {
                    rt.block_on(async_trillion_transactions(batch_size, batches));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_sync_operations,
    bench_async_operations,
    bench_order_flattening,
    bench_hft_simulation,
    bench_trillion_scale
);
criterion_main!(benches);