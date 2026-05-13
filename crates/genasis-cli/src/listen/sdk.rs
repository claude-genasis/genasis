//! v0.6.0 M-v6.0.1 — Claude Agent SDK subprocess 호출.
//!
//! `claude --print` 는 stateless + tool 권한 X 라 agent 가 실제 코드 변경
//! 못 함. Agent SDK (`@anthropic-ai/claude-agent-sdk`) 를 Node subprocess
//! 로 띄워 cwd + Read/Edit/Bash tool 권한을 부여한다. 사용자 §"trial app
//! 시뮬레이션 아닌 실제 동작" 의 본질 기반.
//!
//! 인증은 로컬 Claude Code 세션 자동 검출 — ANTHROPIC_API_KEY 불필요
//! (memory: `feedback_no_claude_api`).

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Node Agent SDK 가 설치된 글로벌 경로. install.sh / 사용자 환경 별로
/// 다를 수 있어 env override 허용.
fn node_path() -> String {
    std::env::var("GENASIS_NODE_PATH")
        .unwrap_or_else(|_| "/home/bravo/.npm-global/lib/node_modules".to_string())
}

/// Agent SDK 호출 — Node subprocess `node -e "<inline>"` 형태로 spawn.
///
/// * `prompt` — agent 에게 보낼 system + user 메시지 통합 텍스트
/// * `cwd` — agent 가 인지할 프로젝트 root (Read/Edit tool 가 여기 기준)
/// * `allowed_tools` — 권한 부여할 tool 목록 (예 `["Read", "Edit", "Bash"]`)
/// * `timeout_secs` — 전체 호출 타임아웃
///
/// 반환은 agent 의 최종 자연어 응답 (assistant message text). tool
/// invocation 메시지나 partial chunk 는 제외.
pub async fn run_claude_agent_sdk(
    prompt: &str,
    cwd: &Path,
    allowed_tools: &[&str],
    timeout_secs: u64,
) -> Result<String> {
    let tools_js = allowed_tools
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(",");

    // Node inline script — SDK 의 query() 를 async for-await 로 소비하고
    // 마지막 assistant text 또는 result 를 stdout 으로 출력.
    let script = format!(
        r#"
const {{ query }} = require('@anthropic-ai/claude-agent-sdk');
(async () => {{
  let lastText = '';
  let result = '';
  try {{
    const stream = query({{
      prompt: process.env.GENASIS_PROMPT,
      options: {{
        permissionMode: 'acceptEdits',
        cwd: process.env.GENASIS_CWD,
        allowedTools: [{tools_js}],
      }}
    }});
    for await (const m of stream) {{
      if (m.type === 'result') {{
        result = m.result || '';
      }} else if (m.type === 'assistant') {{
        const blocks = m.message?.content || [];
        for (const b of blocks) {{
          if (b.type === 'text' && b.text) lastText = b.text;
        }}
      }}
    }}
    process.stdout.write(result || lastText);
  }} catch (e) {{
    process.stderr.write('SDK_ERR: ' + (e?.message || String(e)));
    process.exit(2);
  }}
}})();
"#,
        tools_js = tools_js,
    );

    let mut cmd = Command::new("node");
    cmd.env("NODE_PATH", node_path())
        .env("GENASIS_PROMPT", prompt)
        .env("GENASIS_CWD", cwd.display().to_string())
        .arg("-e")
        .arg(&script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());

    debug!(
        target: "listen",
        cwd = %cwd.display(),
        tools = ?allowed_tools,
        prompt_len = prompt.len(),
        "agent SDK invocation"
    );

    let fut = async {
        let child = cmd.spawn().context("spawn node Agent SDK subprocess")?;
        let out = child.wait_with_output().await.context("wait_with_output")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(anyhow!(
                "agent SDK exited code={:?} stderr={stderr}",
                out.status.code()
            ));
        }
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        Ok::<String, anyhow::Error>(stdout)
    };

    let res = timeout(Duration::from_secs(timeout_secs), fut)
        .await
        .map_err(|_| anyhow!("agent SDK timeout after {timeout_secs}s"))??;

    if res.trim().is_empty() {
        warn!(
            target: "listen",
            "agent SDK returned empty stdout — likely tool-only run"
        );
    }
    Ok(res)
}
