//! SQL read-only lex guard.
//!
//! Inspects the *first significant token* of each top-level statement in the
//! input. The guard is intentionally **conservative** — it errs on the side
//! of rejection. The DB-side enforcement (read-only role / `PRAGMA
//! query_only` / `--readonly` flag) is the second line of defence.
//!
//! Allowed first tokens (case-insensitive):
//!   SELECT, WITH (CTE), EXPLAIN, ANALYZE, DESCRIBE, DESC, SHOW, PRAGMA,
//!   VALUES, TABLE.
//!
//! Forbidden first tokens (case-insensitive):
//!   INSERT, UPDATE, DELETE, DROP, ALTER, CREATE, TRUNCATE, GRANT, REVOKE,
//!   MERGE, REPLACE, CALL, EXEC, EXECUTE, ATTACH, DETACH, COPY, LOAD, INSTALL,
//!   VACUUM, REINDEX, BEGIN, COMMIT, ROLLBACK, SAVEPOINT, SET, RESET, LOCK.
//!
//! Multiple statements are split on **top-level** semicolons (i.e. those
//! outside string literals and comments). Each statement is checked.

use crate::Result;
use genasis_core::Error;

const ALLOWED: &[&str] = &[
    "SELECT", "WITH", "EXPLAIN", "ANALYZE", "DESCRIBE", "DESC", "SHOW", "PRAGMA", "VALUES", "TABLE",
];

const FORBIDDEN: &[&str] = &[
    "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE", "TRUNCATE", "GRANT", "REVOKE",
    "MERGE", "REPLACE", "CALL", "EXEC", "EXECUTE", "ATTACH", "DETACH", "COPY", "LOAD", "INSTALL",
    "VACUUM", "REINDEX", "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT", "SET", "RESET", "LOCK",
];

/// Returns `Ok(())` only if every top-level statement is read-only.
pub fn check_readonly(sql: &str) -> Result<()> {
    let stripped = strip_comments(sql);
    let stmts = split_statements(&stripped);

    let mut any = false;
    for s in stmts {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            continue;
        }
        any = true;
        let token = first_keyword(trimmed);
        let upper = token.to_ascii_uppercase();
        if FORBIDDEN.iter().any(|kw| *kw == upper) {
            return Err(Error::Db(format!(
                "read-only guard: forbidden first token `{token}` in statement: {trimmed:.80}"
            )));
        }
        if !ALLOWED.iter().any(|kw| *kw == upper) {
            return Err(Error::Db(format!(
                "read-only guard: unrecognised first token `{token}` (allowed: {ALLOWED:?})"
            )));
        }
    }

    if !any {
        return Err(Error::Db("read-only guard: empty SQL".into()));
    }
    Ok(())
}

/// Strip `--` line comments and `/* ... */` block comments. Preserves string
/// literals so that comment-like sequences inside strings are not removed.
fn strip_comments(sql: &str) -> String {
    let bytes = sql.as_bytes();
    let mut out = String::with_capacity(sql.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // Single-quoted or double-quoted literal — copy verbatim until closing
        // quote (handle doubled-quote escape).
        if b == b'\'' || b == b'"' {
            let quote = b;
            out.push(b as char);
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                out.push(c as char);
                i += 1;
                if c == quote {
                    if i < bytes.len() && bytes[i] == quote {
                        out.push(quote as char);
                        i += 1;
                        continue;
                    }
                    break;
                }
            }
            continue;
        }
        // -- line comment
        if b == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // /* block comment */
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Split on top-level semicolons (outside string literals).
fn split_statements(sql: &str) -> Vec<&str> {
    let mut stmts = Vec::new();
    let bytes = sql.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\'' || b == b'"' {
            let quote = b;
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                i += 1;
                if c == quote {
                    if i < bytes.len() && bytes[i] == quote {
                        i += 1;
                        continue;
                    }
                    break;
                }
            }
            continue;
        }
        if b == b';' {
            stmts.push(&sql[start..i]);
            start = i + 1;
        }
        i += 1;
    }
    if start < sql.len() {
        stmts.push(&sql[start..]);
    }
    stmts
}

/// The first token (`[A-Za-z_][A-Za-z0-9_]*`) of `s`, ignoring leading
/// whitespace and any leading parentheses (since `( SELECT ... )` is valid).
fn first_keyword(s: &str) -> String {
    let mut it = s.chars().peekable();
    while let Some(&c) = it.peek() {
        if c.is_whitespace() || c == '(' {
            it.next();
        } else {
            break;
        }
    }
    let mut out = String::new();
    while let Some(&c) = it.peek() {
        if c.is_alphanumeric() || c == '_' {
            out.push(c);
            it.next();
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_drop() {
        assert!(check_readonly("DROP TABLE users").is_err());
        assert!(check_readonly("  drop table users").is_err());
    }

    #[test]
    fn rejects_dml() {
        for s in &[
            "INSERT INTO t VALUES (1)",
            "UPDATE t SET a=1",
            "DELETE FROM t WHERE 1=1",
            "TRUNCATE t",
            "MERGE INTO t USING s ON t.id=s.id",
        ] {
            let r = check_readonly(s);
            assert!(r.is_err(), "should reject: {s}");
        }
    }

    #[test]
    fn rejects_ddl_and_dcl() {
        for s in &[
            "CREATE TABLE t (id INT)",
            "ALTER TABLE t ADD COLUMN c INT",
            "GRANT SELECT ON t TO u",
            "REVOKE ALL ON t FROM u",
        ] {
            assert!(check_readonly(s).is_err(), "should reject: {s}");
        }
    }

    #[test]
    fn allows_select_and_friends() {
        for s in &[
            "SELECT 1",
            "  with cte as (select 1) select * from cte",
            "EXPLAIN SELECT * FROM users",
            "ANALYZE TABLE users",
            "DESCRIBE users",
            "DESC users",
            "SHOW TABLES",
            "PRAGMA table_info(users)",
            "VALUES (1), (2)",
            "( SELECT 1 )",
        ] {
            assert!(check_readonly(s).is_ok(), "should allow: {s}");
        }
    }

    #[test]
    fn rejects_chained_statements_when_any_is_write() {
        assert!(check_readonly("SELECT 1; DROP TABLE users").is_err());
        assert!(check_readonly("SELECT 1;\nDELETE FROM x").is_err());
    }

    #[test]
    fn allows_chained_reads() {
        assert!(check_readonly("SELECT 1; SELECT 2; SELECT 3").is_ok());
    }

    #[test]
    fn comments_dont_disguise_intent() {
        assert!(check_readonly("/* hi */ SELECT 1").is_ok());
        assert!(check_readonly("-- hint\nSELECT 1").is_ok());
        // A comment-prefixed write is still a write.
        assert!(check_readonly("/* hi */ DROP TABLE t").is_err());
    }

    #[test]
    fn string_literals_with_semicolons_dont_split() {
        assert!(check_readonly("SELECT 'a;b' FROM t").is_ok());
        // Double-quoted identifier with embedded semicolon.
        assert!(check_readonly("SELECT \"col;name\" FROM t").is_ok());
    }

    #[test]
    fn rejects_unknown_first_token() {
        assert!(check_readonly("FROBNICATE foo").is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(check_readonly("").is_err());
        assert!(check_readonly("   \n  ").is_err());
        assert!(check_readonly("-- only a comment").is_err());
    }

    #[test]
    fn rejects_transaction_control() {
        for s in &["BEGIN", "COMMIT", "ROLLBACK", "SET TRANSACTION READ ONLY"] {
            assert!(check_readonly(s).is_err(), "should reject: {s}");
        }
    }
}
