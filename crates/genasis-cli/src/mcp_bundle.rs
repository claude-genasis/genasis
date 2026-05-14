//! D-057: MCP server scripts + runtime embedded into the binary.
//!
//! Why this module exists: `build_mcp_config` 의 default `mcp_server_dir`
//! 는 이전엔 `env!("CARGO_MANIFEST_DIR")` 의 parent::parent join("mcp-servers")
//! 였는데, 이 매크로는 **컴파일 타임에 박혀**서 release CI 에서 빌드된
//! musl 바이너리는 `/home/runner/work/genasis/genasis/mcp-servers/...` 를
//! 가리킨다. 사용자 머신에는 그 경로가 없어서 `node` 가 `Cannot find
//! module` 으로 죽고, claude session 의 `mcp__trial-app__*` tool 이
//! 등록되지 않아 PM 이 사용자에게 답을 보내지 못한다.
//!
//! 추가 문제: `@modelcontextprotocol/sdk` 가 npm-global 에 없는 환경에선
//! NODE_PATH 가 가리키는 곳에 SDK 가 없어도 같은 증상. 따라서 이 모듈은
//! 두 가지를 함께 책임진다.
//!
//! 1. **embedded .mjs unpack** — `include_str!` 로 바이너리에 박은 3 개
//!    server source 를 `$CACHE/genasis/mcp-servers/<name>/index.mjs` 로
//!    풀어둔다 (hash 일치하면 skip).
//! 2. **lazy npm install** — `$CACHE/genasis/mcp-servers/package.json` 에
//!    `@modelcontextprotocol/sdk` 의존성을 적고, `node_modules/...` 가
//!    아직 없으면 `npm install --prefix $CACHE/genasis/mcp-servers` 를
//!    실행해서 SDK 를 받아둔다 (한 번만; 후속 호출에서는 skip).
//!
//! 결과로 `McpBundle { server_dir, node_modules }` 반환. cmd_listen 이
//! 그걸 받아 build_mcp_config 에 넘기고, 자식 `node` 프로세스는 그
//! NODE_PATH 로 SDK 를 찾는다.
//!
//! `GENASIS_MCP_SERVER_DIR` env 가 설정돼 있으면 이 모듈을 우회하고
//! 그 디렉터리 그대로 쓴다 — 개발 / 디버깅 escape hatch (이 경로
//! 사용자는 자기 환경에서 `npm install` 직접 책임).

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const SDK_VERSION: &str = "^1.0.0";

/// 임베드된 MCP server 본문 (server name, index.mjs source).
const SERVERS: &[(&str, &str)] = &[
    (
        "trial-app",
        include_str!("../../../mcp-servers/trial-app/index.mjs"),
    ),
    (
        "mattermost",
        include_str!("../../../mcp-servers/mattermost/index.mjs"),
    ),
    (
        "plane",
        include_str!("../../../mcp-servers/plane/index.mjs"),
    ),
];

/// 풀어둔 cache 의 두 경로. cmd_listen → session::build_mcp_config 가 둘 다 사용.
pub struct McpBundle {
    /// `<cache>/genasis/mcp-servers/` — 그 하위에 `<name>/index.mjs` 들이 있음.
    pub server_dir: PathBuf,
    /// `<cache>/genasis/mcp-servers/node_modules/` — NODE_PATH 로 자식에게 전달.
    pub node_modules: PathBuf,
}

pub fn ensure_mcp_servers() -> Result<McpBundle> {
    if let Ok(override_dir) = std::env::var("GENASIS_MCP_SERVER_DIR") {
        // override 면 그 디렉터리에 node_modules 가 이미 있다고 가정.
        let p = PathBuf::from(override_dir);
        let nm = p.join("node_modules");
        return Ok(McpBundle {
            server_dir: p,
            node_modules: nm,
        });
    }
    let cache_root = dirs::cache_dir()
        .ok_or_else(|| anyhow!("no cache dir — set GENASIS_MCP_SERVER_DIR explicitly"))?
        .join("genasis")
        .join("mcp-servers");
    fs::create_dir_all(&cache_root)
        .with_context(|| format!("creating MCP cache dir {}", cache_root.display()))?;

    // 1. embedded .mjs unpack (hash skip).
    for (name, source) in SERVERS {
        let server_dir = cache_root.join(name);
        fs::create_dir_all(&server_dir)
            .with_context(|| format!("creating {}", server_dir.display()))?;
        let index_path = server_dir.join("index.mjs");
        let want_hash = sha256(source.as_bytes());
        let stale = match fs::read(&index_path) {
            Ok(existing) => sha256(&existing) != want_hash,
            Err(_) => true,
        };
        if stale {
            fs::write(&index_path, source)
                .with_context(|| format!("writing {}", index_path.display()))?;
        }
    }

    // 2. cache root 의 package.json — SDK dep 선언.
    let pkg_path = cache_root.join("package.json");
    let pkg_json = format!(
        r#"{{
  "name": "@genasis/mcp-runtime",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "dependencies": {{
    "@modelcontextprotocol/sdk": "{SDK_VERSION}"
  }}
}}
"#
    );
    let pkg_stale = match fs::read_to_string(&pkg_path) {
        Ok(existing) => existing.trim() != pkg_json.trim(),
        Err(_) => true,
    };
    if pkg_stale {
        fs::write(&pkg_path, &pkg_json)
            .with_context(|| format!("writing {}", pkg_path.display()))?;
    }

    // 3. node_modules/@modelcontextprotocol/sdk 존재 확인 — 없으면 npm install.
    let node_modules = cache_root.join("node_modules");
    let sdk_marker = node_modules.join("@modelcontextprotocol").join("sdk");
    if !sdk_marker.is_dir() {
        eprintln!(
            "[..] preparing MCP runtime in {} (first time only, ~10-30s)...",
            cache_root.display()
        );
        run_npm_install(&cache_root)?;
        if !sdk_marker.is_dir() {
            return Err(anyhow!(
                "npm install completed but {} still missing — check node + npm versions",
                sdk_marker.display()
            ));
        }
        eprintln!("[OK] MCP runtime ready.");
    }

    Ok(McpBundle {
        server_dir: cache_root,
        node_modules,
    })
}

fn run_npm_install(cache_root: &Path) -> Result<()> {
    let status = std::process::Command::new("npm")
        .args([
            "install",
            "--prefix",
            cache_root.to_str().unwrap_or("."),
            "--no-audit",
            "--no-fund",
            "--silent",
        ])
        .status()
        .with_context(|| {
            "running `npm install` — make sure `node` + `npm` are on PATH (install.sh \
             checks for them; rerun `genasis doctor` if missing)"
        })?;
    if !status.success() {
        return Err(anyhow!(
            "npm install failed (exit {:?}) — see stderr above; try manually running \
             `npm install --prefix {}`",
            status.code(),
            cache_root.display()
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
