//! `genasis listen start/stop/status/restart/logs` — genesis §28 의
//! `scripts/bridgectl.sh` 등가물.
//!
//! - **PID 파일**: `<project>/.genasis/listen.pid`.
//! - **로그**: `<project>/.genasis/listen.log` (foreground 모드에서는
//!   stderr 도 같이).
//! - **slug 당 1 프로세스**: 기존 PID 파일이 살아있으면 `start` 거부.
//! - **고아 정리**: PID 파일이 없어도 `/proc/<pid>/cmdline` 에 `genasis
//!   listen` 이 박혀 있고 cwd 가 같으면 잡아서 정리.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PID_FILE_NAME: &str = "listen.pid";
const LOG_FILE_NAME: &str = "listen.log";

pub fn pid_path(project_root: &Path) -> PathBuf {
    project_root.join(".genasis").join(PID_FILE_NAME)
}

pub fn log_path(project_root: &Path) -> PathBuf {
    project_root.join(".genasis").join(LOG_FILE_NAME)
}

pub fn ensure_genasis_dir(project_root: &Path) -> Result<()> {
    fs::create_dir_all(project_root.join(".genasis"))
        .with_context(|| format!("mkdir {}", project_root.join(".genasis").display()))?;
    Ok(())
}

pub fn read_pid(project_root: &Path) -> Result<Option<u32>> {
    let p = pid_path(project_root);
    if !p.is_file() {
        return Ok(None);
    }
    let s = fs::read_to_string(&p)?;
    let pid: u32 = s
        .trim()
        .parse()
        .with_context(|| format!("PID file {} not numeric: {s:?}", p.display()))?;
    Ok(Some(pid))
}

pub fn write_pid(project_root: &Path, pid: u32) -> Result<()> {
    ensure_genasis_dir(project_root)?;
    fs::write(pid_path(project_root), pid.to_string())?;
    Ok(())
}

pub fn remove_pid_file(project_root: &Path) {
    let _ = fs::remove_file(pid_path(project_root));
}

/// PID 가 살아있고 그 프로세스가 진짜 `genasis listen` 인지 검사.
pub fn pid_is_listen(pid: u32) -> bool {
    let cmdline_path = format!("/proc/{pid}/cmdline");
    let raw = match fs::read(&cmdline_path) {
        Ok(b) => b,
        Err(_) => return false, // 프로세스 자체가 없음
    };
    // cmdline 은 NUL-separated arguments
    let s = String::from_utf8_lossy(&raw);
    s.contains("genasis") && s.contains("listen")
}

/// `start` 전 사전 점검: 기존에 살아있는 listen 이 있는지.
pub enum StartPrecheck {
    /// 깨끗함 — 그대로 진행.
    Clean,
    /// 살아있는 listen 발견. PID 보유. 사용자가 stop 호출하라고 거부.
    AlreadyRunning(u32),
    /// PID 파일은 있지만 프로세스는 죽은 stale 상태 — 정리하고 진행.
    StalePid,
}

pub fn start_precheck(project_root: &Path) -> Result<StartPrecheck> {
    let Some(pid) = read_pid(project_root)? else {
        return Ok(StartPrecheck::Clean);
    };
    if pid_is_listen(pid) {
        Ok(StartPrecheck::AlreadyRunning(pid))
    } else {
        Ok(StartPrecheck::StalePid)
    }
}

/// `genasis listen stop` 본체. PID 파일이 가리키는 listen 에 SIGTERM →
/// 3 초 대기 → 안 죽으면 SIGKILL. 어느 쪽이든 PID 파일 정리.
pub fn stop_daemon(project_root: &Path) -> Result<()> {
    let pid = match read_pid(project_root)? {
        Some(p) => p,
        None => {
            println!("listen: PID 파일 없음 — 이미 중지된 상태");
            return Ok(());
        }
    };
    if !pid_is_listen(pid) {
        println!("listen: PID {pid} 가 더는 listen 이 아님 — stale PID 파일 정리");
        remove_pid_file(project_root);
        return Ok(());
    }
    // SIGTERM
    let pid_i = pid as i32;
    unsafe {
        if libc_kill(pid_i, 15) != 0 {
            return Err(anyhow!("SIGTERM to {pid} failed"));
        }
    }
    println!("listen: SIGTERM → PID {pid}, 3 초 대기…");
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        if !pid_is_listen(pid) {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    if pid_is_listen(pid) {
        println!("listen: 3 초 후에도 살아있음 → SIGKILL");
        unsafe {
            libc_kill(pid_i, 9);
        }
    }
    remove_pid_file(project_root);
    println!("listen: 중지 완료");
    Ok(())
}

pub fn status(project_root: &Path) -> Result<()> {
    let pid = match read_pid(project_root)? {
        Some(p) => p,
        None => {
            println!("listen: ❌ 실행 중 아님 (PID 파일 없음)");
            return Ok(());
        }
    };
    if !pid_is_listen(pid) {
        println!("listen: ⚠️ PID 파일은 있지만 프로세스 ({pid}) 가 없음 — `genasis listen start` 로 재시작 가능");
        return Ok(());
    }
    println!("listen: ✅ 실행 중 (PID {pid})");
    let log = log_path(project_root);
    if log.is_file() {
        println!("\n--- 최근 로그 3 줄 ---");
        let s = fs::read_to_string(&log).unwrap_or_default();
        for line in s.lines().rev().take(3).collect::<Vec<_>>().iter().rev() {
            println!("  {line}");
        }
    }
    Ok(())
}

/// minimal kill(2) wrapper — libc 가져오기 무거우므로 직접 syscall.
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

unsafe fn libc_kill(pid: i32, sig: i32) -> i32 {
    unsafe { kill(pid, sig) }
}
