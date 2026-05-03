//! Cross-crate test: SQL read-only guard catches the documented forbidden
//! tokens and accepts the documented read-only tokens.

use genasis_db::guard::check_readonly;

#[test]
fn allows_canonical_reads() {
    let cases = [
        "SELECT * FROM users",
        "  with cte as (select 1) select * from cte",
        "EXPLAIN SELECT 1",
        "ANALYZE TABLE users",
        "DESCRIBE users",
        "DESC users",
        "SHOW TABLES",
        "PRAGMA table_info(users)",
    ];
    for sql in cases {
        assert!(check_readonly(sql).is_ok(), "should allow: {sql}");
    }
}

#[test]
fn rejects_writes_and_dcl_and_tx() {
    let cases = [
        "INSERT INTO t VALUES (1)",
        "UPDATE t SET a=1",
        "DELETE FROM t",
        "DROP TABLE t",
        "ALTER TABLE t ADD c INT",
        "CREATE TABLE t (id INT)",
        "TRUNCATE t",
        "GRANT SELECT ON t TO u",
        "REVOKE ALL ON t FROM u",
        "CALL p()",
        "EXEC p",
        "ATTACH DATABASE 'x.db' AS x",
        "BEGIN",
        "COMMIT",
        "ROLLBACK",
        "SET TRANSACTION READ ONLY",
        "VACUUM",
    ];
    for sql in cases {
        assert!(check_readonly(sql).is_err(), "should reject: {sql}");
    }
}

#[test]
fn rejects_chained_with_any_write() {
    assert!(check_readonly("SELECT 1; DROP TABLE u").is_err());
    assert!(check_readonly("SELECT 1; INSERT INTO t VALUES (1)").is_err());
}

#[test]
fn comments_dont_disguise_writes() {
    assert!(check_readonly("/* nice */ SELECT 1").is_ok());
    assert!(check_readonly("-- hint\nSELECT 1").is_ok());
    assert!(check_readonly("/* hi */ DROP TABLE t").is_err());
    assert!(check_readonly("-- look ma\nUPDATE t SET a=1").is_err());
}

#[test]
fn semicolons_inside_strings_are_safe() {
    assert!(check_readonly("SELECT 'a; b' FROM t").is_ok());
    assert!(check_readonly("SELECT \"col;name\" FROM t").is_ok());
}
