#!/usr/bin/env sh
# =============================================================================
# Genasis launcher — downloads the matching pre-built binary from GitHub
# Releases. Does NOT compile from source. Contributors should clone the repo
# and run `cargo build --release` instead.
#
# Pipeline:
#   1. Detect OS / arch / distro.
#   2. Probe for required and optional packages.
#   3. Print OS-specific install commands for whatever is missing.
#   4. Resolve the release asset URL, download, verify sha256, extract.
#   5. Install to ~/.local/bin/genasis (fallback: /usr/local/bin/genasis with sudo).
#   6. Optionally exec `genasis attach`.
#
# Flags:
#   --no-run                Skip the auto `genasis attach` at the end.
#   --prefix=PATH           Override install dir (default: ~/.local/bin).
#   --version=vX.Y.Z        Pin a specific release (default: latest).
#   --skip-prereqs          Bypass prerequisite check (not recommended).
#
# OWNER must be replaced with the actual GitHub owner before publishing.
# =============================================================================
set -eu

OWNER="${GENASIS_OWNER:-OWNER}"
REPO="genasis"
RELEASE_VERSION="latest"
PREFIX=""
RUN_AFTER_INSTALL=1
SKIP_PREREQS=0

# ---- parse args -------------------------------------------------------------
for arg in "$@"; do
    case "$arg" in
        --no-run) RUN_AFTER_INSTALL=0 ;;
        --skip-prereqs) SKIP_PREREQS=1 ;;
        --prefix=*) PREFIX="${arg#--prefix=}" ;;
        --version=*) RELEASE_VERSION="${arg#--version=}" ;;
        -h|--help)
            cat <<HELP
Genasis installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/${OWNER}/${REPO}/main/install.sh | sh
  curl -fsSL .../install.sh | sh -s -- --no-run --version=v0.1.0

Flags:
  --no-run             Skip the auto attach run.
  --prefix=PATH        Override install dir (default: ~/.local/bin).
  --version=vX.Y.Z     Pin a release (default: latest).
  --skip-prereqs       Bypass prerequisite check.
HELP
            exit 0
            ;;
        *) echo "unknown flag: $arg" >&2; exit 2 ;;
    esac
done

# ---- pretty printing --------------------------------------------------------
if [ -t 1 ]; then
    C_RED='\033[0;31m'; C_GRN='\033[0;32m'; C_YLW='\033[1;33m'
    C_BLU='\033[0;34m'; C_DIM='\033[2m'; C_RST='\033[0m'
else
    C_RED=""; C_GRN=""; C_YLW=""; C_BLU=""; C_DIM=""; C_RST=""
fi
info()  { printf "%b[..]%b %s\n" "$C_BLU" "$C_RST" "$*"; }
ok()    { printf "%b[OK]%b %s\n" "$C_GRN" "$C_RST" "$*"; }
warn()  { printf "%b[WARN]%b %s\n" "$C_YLW" "$C_RST" "$*" >&2; }
err()   { printf "%b[ERR]%b %s\n" "$C_RED" "$C_RST" "$*" >&2; }
die()   { err "$*"; exit 1; }
hr()    { printf "%b%s%b\n" "$C_DIM" "----------------------------------------------------------------" "$C_RST"; }

# ---- detect OS / arch / distro ---------------------------------------------
detect_platform() {
    OS_KERNEL="$(uname -s)"
    ARCH_RAW="$(uname -m)"

    case "$OS_KERNEL" in
        Linux) OS="linux" ;;
        Darwin) OS="macos" ;;
        MINGW*|MSYS*|CYGWIN*)
            die "Windows native is not supported. Please run inside WSL2 (Ubuntu recommended) and re-execute this installer."
            ;;
        *) die "unsupported kernel: $OS_KERNEL" ;;
    esac

    case "$ARCH_RAW" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) die "unsupported architecture: $ARCH_RAW" ;;
    esac

    DISTRO=""
    DISTRO_LIKE=""
    PKG_MGR=""
    if [ "$OS" = "linux" ] && [ -r /etc/os-release ]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        DISTRO="${ID:-}"
        DISTRO_LIKE="${ID_LIKE:-}"
        case "$DISTRO" in
            ubuntu|debian|linuxmint|pop) PKG_MGR="apt" ;;
            fedora|rhel|centos|rocky|almalinux) PKG_MGR="dnf" ;;
            arch|manjaro|endeavouros) PKG_MGR="pacman" ;;
            opensuse*|sles) PKG_MGR="zypper" ;;
            alpine) PKG_MGR="apk" ;;
            *)
                case "$DISTRO_LIKE" in
                    *debian*) PKG_MGR="apt" ;;
                    *rhel*|*fedora*) PKG_MGR="dnf" ;;
                    *arch*) PKG_MGR="pacman" ;;
                    *suse*) PKG_MGR="zypper" ;;
                esac
                ;;
        esac
    elif [ "$OS" = "macos" ]; then
        if command -v brew >/dev/null 2>&1; then PKG_MGR="brew"
        elif command -v port >/dev/null 2>&1; then PKG_MGR="port"
        else PKG_MGR=""
        fi
    fi

    info "Platform: $OS-$ARCH${DISTRO:+  distro=$DISTRO}${PKG_MGR:+  pkg=$PKG_MGR}"
}

# ---- prerequisite probes ----------------------------------------------------
# Each line: NAME|REQUIRED|PURPOSE
PREREQS="
git|required|sources & contributor workflows
curl|required|installer self
tar|required|extracting release archives
bash|required|hooks & scripts
node|optional|Plane user provisioning (Playwright sub-process); requires >=18
gh|optional|GitHub branch protection / PR automation
atlas|optional|DB schema migrations (postgres/mysql/sqlite)
psql|optional|DB read-only queries when driver=postgres
mysql|optional|DB read-only queries when driver=mysql
sqlite3|optional|DB read-only queries when driver=sqlite
duckdb|optional|DB read-only queries when driver=duckdb
rtk|optional|token savings on shell tool calls
claude|optional|Claude Code CLI for the agentic team itself
"

probe_one() {
    cmd="$1"
    case "$cmd" in
        node) command -v node >/dev/null 2>&1 || return 1
              ver="$(node --version 2>/dev/null | sed 's/^v//' | cut -d. -f1)"
              [ "${ver:-0}" -ge 18 ] || return 1
              return 0
              ;;
        *) command -v "$cmd" >/dev/null 2>&1 ;;
    esac
}

# Print install command per package per OS/distro.
# stdout: a multiline string the caller will indent.
install_hint() {
    pkg="$1"
    case "$pkg|$OS|$PKG_MGR" in
        git\|linux\|apt)      echo "sudo apt update && sudo apt install -y git" ;;
        git\|linux\|dnf)      echo "sudo dnf install -y git" ;;
        git\|linux\|pacman)   echo "sudo pacman -S --noconfirm git" ;;
        git\|linux\|zypper)   echo "sudo zypper install -y git" ;;
        git\|linux\|apk)      echo "sudo apk add git" ;;
        git\|macos\|brew)     echo "brew install git" ;;
        git\|macos\|port)     echo "sudo port install git" ;;

        curl\|linux\|apt)     echo "sudo apt install -y curl" ;;
        curl\|linux\|dnf)     echo "sudo dnf install -y curl" ;;
        curl\|linux\|pacman)  echo "sudo pacman -S --noconfirm curl" ;;
        curl\|linux\|apk)     echo "sudo apk add curl" ;;
        curl\|macos\|brew)    echo "brew install curl" ;;

        tar\|linux\|apt)      echo "sudo apt install -y tar" ;;
        tar\|linux\|dnf)      echo "sudo dnf install -y tar" ;;
        tar\|linux\|pacman)   echo "sudo pacman -S --noconfirm tar" ;;
        tar\|linux\|apk)      echo "sudo apk add tar" ;;

        bash\|linux\|apt)     echo "sudo apt install -y bash" ;;
        bash\|linux\|dnf)     echo "sudo dnf install -y bash" ;;
        bash\|linux\|apk)     echo "sudo apk add bash" ;;

        node\|linux\|apt)     echo "sudo apt install -y nodejs npm   # may be too old"
                              echo "# or via nvm:"
                              echo "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash && nvm install --lts" ;;
        node\|linux\|dnf)     echo "sudo dnf install -y nodejs npm" ;;
        node\|linux\|pacman)  echo "sudo pacman -S --noconfirm nodejs npm" ;;
        node\|linux\|zypper)  echo "sudo zypper install -y nodejs20 npm20" ;;
        node\|linux\|apk)     echo "sudo apk add nodejs npm" ;;
        node\|macos\|brew)    echo "brew install node" ;;
        node\|macos\|port)    echo "sudo port install nodejs20" ;;

        gh\|linux\|apt)       echo "https://github.com/cli/cli/blob/trunk/docs/install_linux.md  (apt repo)" ;;
        gh\|linux\|dnf)       echo "sudo dnf install -y gh" ;;
        gh\|linux\|pacman)    echo "sudo pacman -S --noconfirm github-cli" ;;
        gh\|linux\|apk)       echo "sudo apk add github-cli" ;;
        gh\|macos\|brew)      echo "brew install gh" ;;
        gh\|macos\|port)      echo "sudo port install gh" ;;

        atlas\|*\|*)          echo "curl -sSf https://atlasgo.sh | sh   # https://atlasgo.io/getting-started" ;;

        psql\|linux\|apt)     echo "sudo apt install -y postgresql-client" ;;
        psql\|linux\|dnf)     echo "sudo dnf install -y postgresql" ;;
        psql\|linux\|pacman)  echo "sudo pacman -S --noconfirm postgresql" ;;
        psql\|linux\|apk)     echo "sudo apk add postgresql-client" ;;
        psql\|macos\|brew)    echo "brew install libpq && brew link --force libpq" ;;

        mysql\|linux\|apt)    echo "sudo apt install -y default-mysql-client" ;;
        mysql\|linux\|dnf)    echo "sudo dnf install -y mysql" ;;
        mysql\|linux\|pacman) echo "sudo pacman -S --noconfirm mariadb-clients" ;;
        mysql\|linux\|apk)    echo "sudo apk add mysql-client" ;;
        mysql\|macos\|brew)   echo "brew install mysql-client" ;;

        sqlite3\|linux\|apt)  echo "sudo apt install -y sqlite3" ;;
        sqlite3\|linux\|dnf)  echo "sudo dnf install -y sqlite" ;;
        sqlite3\|linux\|pacman) echo "sudo pacman -S --noconfirm sqlite" ;;
        sqlite3\|linux\|apk)  echo "sudo apk add sqlite" ;;
        sqlite3\|macos\|brew) echo "brew install sqlite" ;;

        duckdb\|linux\|*)     echo "https://duckdb.org/docs/installation/  (download static binary; pacman: 'duckdb' AUR; brew: 'brew install duckdb')" ;;
        duckdb\|macos\|brew)  echo "brew install duckdb" ;;

        rtk\|*\|*)            echo "https://github.com/anthropic-experimental/rtk  (cargo install rtk-cli  or  download release binary)" ;;

        claude\|*\|*)         echo "https://docs.anthropic.com/claude/docs/claude-code  (npm i -g @anthropic-ai/claude-code  or platform installer)" ;;

        *) echo "see upstream docs for $pkg on $OS ($PKG_MGR)" ;;
    esac
}

run_prereq_check() {
    [ "$SKIP_PREREQS" -eq 1 ] && { warn "Skipping prerequisite check (--skip-prereqs)."; return; }

    info "Checking prerequisites..."
    hr
    missing_required=""
    missing_optional=""

    # Iterate while preserving spaces in the purpose field.
    echo "$PREREQS" | while IFS='|' read -r pkg req purpose; do
        [ -z "${pkg:-}" ] && continue
        if probe_one "$pkg"; then
            ok "$pkg ($req) — found"
        else
            if [ "$req" = "required" ]; then
                err "$pkg (required) — MISSING. Purpose: $purpose"
            else
                warn "$pkg (optional) — missing. Purpose: $purpose"
            fi
            printf "    %bsuggested install command:%b\n" "$C_DIM" "$C_RST"
            install_hint "$pkg" | sed 's/^/      /'
        fi
    done
    hr

    # Re-evaluate required misses outside the subshell.
    final_missing=""
    for pkg in git curl tar bash; do
        probe_one "$pkg" || final_missing="$final_missing $pkg"
    done
    if [ -n "$final_missing" ]; then
        die "missing required tools:$final_missing — install them and re-run."
    fi
    ok "all required prerequisites present."
}

# ---- resolve install dir ----------------------------------------------------
resolve_prefix() {
    if [ -n "$PREFIX" ]; then return; fi
    if [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
        PREFIX="$HOME/.local/bin"
    else
        PREFIX="/usr/local/bin"
    fi
    info "Install prefix: $PREFIX"
}

# ---- download release asset ------------------------------------------------
fetch_binary() {
    asset="genasis-${OS}-${ARCH}.tar.gz"
    if [ "$RELEASE_VERSION" = "latest" ]; then
        url_base="https://github.com/${OWNER}/${REPO}/releases/latest/download"
    else
        url_base="https://github.com/${OWNER}/${REPO}/releases/download/${RELEASE_VERSION}"
    fi
    asset_url="${url_base}/${asset}"
    sha_url="${url_base}/${asset}.sha256"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    info "Downloading $asset_url"
    if ! curl -fSL --progress-bar -o "$tmp/$asset" "$asset_url"; then
        warn "release asset not yet available (M0 stage)."
        warn "  url tried: $asset_url"
        warn "  this is expected before the first cross-compile / release upload."
        warn "  contributors can build from source: cargo build --release"
        return 1
    fi

    info "Verifying sha256..."
    if curl -fSL -o "$tmp/$asset.sha256" "$sha_url" 2>/dev/null; then
        expected="$(cut -d' ' -f1 "$tmp/$asset.sha256")"
        actual="$(sha256sum "$tmp/$asset" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$tmp/$asset" | cut -d' ' -f1)"
        if [ "$expected" != "$actual" ]; then
            die "sha256 mismatch: expected=$expected actual=$actual"
        fi
        ok "sha256 verified."
    else
        warn "sha256 file missing — proceeding without verification (not recommended)."
    fi

    info "Extracting..."
    tar -xzf "$tmp/$asset" -C "$tmp"

    if [ ! -f "$tmp/genasis" ]; then
        die "binary not found in archive (expected: ./genasis)"
    fi

    install_path="$PREFIX/genasis"
    if mv "$tmp/genasis" "$install_path" 2>/dev/null; then
        chmod +x "$install_path"
    else
        warn "cannot write to $PREFIX without elevation; trying sudo."
        sudo install -m 0755 "$tmp/genasis" "$install_path"
    fi
    ok "Installed: $install_path"

    case ":${PATH:-}:" in
        *":$PREFIX:"*) : ;;
        *)
            warn "$PREFIX is not on your PATH. Add to your shell rc:"
            printf "    %bexport PATH=\"%s:\$PATH\"%b\n" "$C_DIM" "$PREFIX" "$C_RST"
            ;;
    esac
    return 0
}

# ---- main -------------------------------------------------------------------
main() {
    printf "%b" "
  ___                       _
 / __|___ _ _  __ _ _ _ ___(_)___
| (_ / -_) ' \/ _\` | ' \/ _ \ (_-<
 \___\___|_||_\__,_|_||_\___/_/__/
"
    printf "  bolt-on agentic team layer\n"
    hr
    detect_platform
    run_prereq_check
    resolve_prefix

    if fetch_binary; then
        if [ "$RUN_AFTER_INSTALL" -eq 1 ]; then
            info "Running 'genasis attach' (use --no-run to skip)"
            "$PREFIX/genasis" attach || warn "'genasis attach' exited non-zero — check output above."
        else
            ok "Skipping auto-run (--no-run)."
        fi
    else
        warn "Binary install skipped. Either build from source or wait for the first release."
    fi

    hr
    ok "Done. Next steps:"
    printf "  - run %bgenasis doctor%b to re-check prerequisites\n" "$C_BLU" "$C_RST"
    printf "  - run %bgenasis attach%b in your project directory\n" "$C_BLU" "$C_RST"
    printf "  - read %bblueprint.md%b and %bprogress.md%b in the repo for the design\n" "$C_BLU" "$C_RST" "$C_BLU" "$C_RST"
}

main "$@"
