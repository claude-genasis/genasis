//! D-130 verification helper — dump JSONL aggregation results so the
//! CLAUDE USAGE widget's inputs can be sanity-checked from the CLI
//! without driving the ratatui UI. Sister to `detect_dump.rs`.
//!
//! Run with: `cargo run -p genasis-monitor --example usage_dump`.

fn main() {
    let snap = genasis_monitor::collector::jsonl::scan_sessions_dir();
    println!("files_scanned: {}", snap.files_scanned);
    println!("five_h_input: {}", snap.five_h_input_tokens);
    println!("five_h_output: {}", snap.five_h_output_tokens);
    println!("five_h_cache_read: {}", snap.five_h_cache_read);
    println!("five_h_cache_create: {}", snap.five_h_cache_create);
    println!("five_h_cost_usd: ${:.4}", snap.five_h_cost_usd);
    println!("five_h_reset_epoch: {}", snap.five_h_reset_epoch);
    println!("five_h_reset_countdown: {}s", snap.five_h_reset_countdown());
    println!("five_h_oldest_event_ts: {}", snap.five_h_oldest_event_ts);
    println!("five_h_event_count: {}", snap.five_h_event_count);
    println!("is_empty_5h: {}", snap.is_empty_5h());
    println!("week_input: {}", snap.week_input_tokens);
    println!("week_output: {}", snap.week_output_tokens);
    println!("week_cost_usd: ${:.4}", snap.week_cost_usd);
    println!("week_sonnet_input: {}", snap.week_sonnet_input);
    println!("week_sonnet_output: {}", snap.week_sonnet_output);
    println!("week_opus_input: {}", snap.week_opus_input);
    println!("week_opus_output: {}", snap.week_opus_output);
    println!("five_h_window_start: {}", snap.five_h_window_start);
    println!("mcp_calls_5h: {}", snap.mcp_calls_5h);
    let (plan, tier) = genasis_monitor::collector::jsonl::read_credentials();
    println!("creds plan: {}", plan);
    println!("creds tier: {}", tier);
    if let Some((five, all, opus, sonnet)) =
        genasis_monitor::collector::jsonl::tier_to_limits(&tier)
    {
        println!(
            "tier_to_limits: 5h={} week_all={} week_opus={} week_sonnet={}",
            five, all, opus, sonnet
        );
    } else {
        println!("tier_to_limits: <unknown tier — env defaults will be used>");
    }

    // D-131: live OAuth fetch.
    println!();
    println!("=== /api/oauth/usage ===");
    let rt = tokio::runtime::Runtime::new().expect("tokio rt");
    rt.block_on(async {
        match genasis_monitor::collector::oauth_usage::fetch().await {
            Ok(Some(u)) => {
                println!("five_hour          : {:?}", u.five_hour);
                println!("seven_day          : {:?}", u.seven_day);
                println!("seven_day_opus     : {:?}", u.seven_day_opus);
                println!("seven_day_sonnet   : {:?}", u.seven_day_sonnet);
                println!("seven_day_omelette : {:?}", u.seven_day_omelette);
                println!("extra_usage        : {:?}", u.extra_usage);
            }
            Ok(None) => {
                println!("<oauth fetch returned None — token missing/expired or feature disabled>")
            }
            Err(e) => println!("<oauth fetch error: {e}>"),
        }
    });
}
