//! D-099 검증 도우미 — detect_sessions 의 결과를 stdout 으로 dump
//! 해서 monitor TUI 안에서 보기 어려운 분류 결과를 확인한다.

use std::path::Path;

fn main() {
    let sessions = genasis_monitor::collector::sessions::detect_sessions(Path::new(""), "");
    println!("=== {} claude sessions detected ===", sessions.len());
    for s in &sessions {
        println!(
            "PID={:<8} role={:<14} cwd={:<60} age={:<6}s state={:?}",
            s.pid,
            s.role.as_deref().unwrap_or("(none)"),
            s.cwd,
            s.age_secs,
            s.state
        );
    }
}
