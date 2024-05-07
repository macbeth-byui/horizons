#![forbid(unsafe_code)]

use horizons::manager;

#[tokio::main]
async fn main() {
    manager::run().await;
    println!("Goodbye!");
    println!();
}
