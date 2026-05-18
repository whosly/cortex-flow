//! # AI Agent Scheduler
//!
//! AI智能体调度框架
//!
//! ## 运行示例
//!
//! ```bash
//! # 简单示例
//! cargo run --example simple_example
//!
//! # 基本示例（带 tracing 和 metrics）
//! cargo run --example basic_example
//!
//! # 并行执行示例
//! cargo run --example parallel_example
//! ```

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== AI Agent Scheduler ===\n");
    println!("This is a library crate. Run examples with:");
    println!();
    println!("  cargo run --example simple_example");
    println!("  cargo run --example basic_example");
    println!("  cargo run --example parallel_example");
    println!();
    println!("Or run tests:");
    println!("  cargo test");
    println!();

    Ok(())
}
