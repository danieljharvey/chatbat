mod shared;
mod task_planner;

#[tokio::main]
async fn main() {
    task_planner::main().await
}
