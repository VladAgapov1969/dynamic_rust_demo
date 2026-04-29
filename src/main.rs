use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

#[tokio::main]
async fn main() {
    println!("{}", "🚀 Rust Async + Ownership + Multithreading + indicatif Demo".cyan().bold());
    println!("{}", "─────────────────────────────────────────────────────────────".dimmed());

    // 🔑 Arc = Shared ownership across tasks
    // 🔒 RwLock = Safe concurrent read/write access
    let metrics = Arc::new(RwLock::new(TaskMetrics::new()));

    // 📊 MultiProgress coordinates multiple bars — keep it alive for the whole run
    let mp = MultiProgress::new();

    // Create individual progress bars for each task
    let mut task_bars: Vec<ProgressBar> = Vec::new();
    for id in 1..=4 {
        let pb = mp.add(ProgressBar::new(3)); // 3 stages: I/O → CPU → Commit
        pb.set_style(
            ProgressStyle::with_template(
                &format!("{{spinner:.green}} [Task {}] {{elapsed_precise}} {{bar:30.cyan/blue}} {{pos}}/3 {{msg}}", id)
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Initializing...");
        task_bars.push(pb);
    }

    // 📊 Overall progress bar
    let overall_pb = mp.add(ProgressBar::new(4));
    overall_pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}"
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    overall_pb.set_message("Processing tasks...");

    // Spawn 4 async tasks, each with shared access to state and progress bars
    let handles: Vec<_> = (1..=4).map(|id| {
        let state = Arc::clone(&metrics);
        let task_pb = task_bars[id - 1].clone(); // ProgressBar is Clone
        let overall_pb = overall_pb.clone();
        
        tokio::spawn(async move {
            process_task(id, state, task_pb, overall_pb).await;
        })
    }).collect();

    // Wait for all spawned tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Finalize overall progress bar
    overall_pb.finish_with_message("✅ All tasks completed!");

    // Ensure all task bars are finished (safety net)
    for pb in &task_bars {
        if !pb.is_finished() {
            pb.finish_with_message("✅ Complete");
        }
    }

    // Small delay to let MultiProgress render final state before exit
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 📋 Print final aggregated results
    let final_metrics = metrics.read().await;
    println!("\n{}", "📋 Final Results:".cyan().bold());
    for (i, result) in final_metrics.completed.iter().enumerate() {
        println!("  {} Task {}: {}", "✓".green(), i + 1, result);
    }
}

#[derive(Debug, Clone)]
struct TaskMetrics {
    completed: Vec<String>,
}

impl TaskMetrics {
    fn new() -> Self {
        Self { completed: vec![] }
    }
}

async fn process_task(
    id: usize,
    state: Arc<RwLock<TaskMetrics>>,
    pb: ProgressBar,
    overall_pb: ProgressBar,
) {
    // Stage 1: Async I/O simulation
    pb.set_message("📡 Async I/O...");
    tokio::time::sleep(Duration::from_millis(200 + (id as u64 * 50))).await;
    pb.inc(1);
    pb.set_message("📥 I/O complete");

    // Stage 2: CPU-bound work
    pb.set_message("⚙️ CPU processing...");
    let data = format!("RawData-{}", id);
    let data_clone = data.clone();
    
    let processed = tokio::task::spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(300));
        format!("{}_PROCESSED", data_clone)
    })
    .await
    .unwrap_or_else(|_| format!("{}_ERROR", data)); // ← Original `data` still valid

    pb.inc(1);
    pb.set_message("✅ CPU done");

    // Stage 3: Commit result
    pb.set_message("🔒 Committing result...");
    {
        let mut m = state.write().await;
        m.completed.push(processed.clone());
    }
    pb.inc(1);
    
    // ✅ Fix: Pass owned String, not &String
    pb.finish_with_message(format!("✅ Task {} complete: {}", id, processed));

    overall_pb.inc(1);
}