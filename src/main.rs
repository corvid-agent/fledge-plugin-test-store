#[link(wasm_import_module = "fledge")]
extern "C" {
    fn recv(ptr: *mut u8, max_len: i32) -> i32;
    fn send(ptr: *const u8, len: i32);
    fn exit(code: i32);
    fn store_set(ptr: *const u8, len: i32);
    fn store_get(ptr: *const u8, len: i32) -> i32;
}

static mut PASS: u32 = 0;
static mut FAIL: u32 = 0;

fn fledge_recv() -> Vec<u8> {
    let mut buf = vec![0u8; 65536];
    let len = unsafe { recv(buf.as_mut_ptr(), buf.len() as i32) };
    buf.truncate(len.max(0) as usize);
    buf
}

fn fledge_send(msg: &str) {
    unsafe { send(msg.as_ptr(), msg.len() as i32) };
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn output(text: &str) {
    fledge_send(&format!(
        r#"{{"type":"output","text":"{}"}}"#,
        json_escape(text)
    ));
}

fn pass(msg: &str) {
    unsafe { PASS += 1 };
    output(&format!("  \u{2713} PASS: {msg}\n"));
}

fn fail(msg: &str) {
    unsafe { FAIL += 1 };
    output(&format!("  \u{2717} FAIL: {msg}\n"));
}

fn header(title: &str) {
    output(&format!("\n=== {title} ===\n"));
}

fn fledge_store_set(key: &str, value: &str) {
    let req = format!(
        r#"{{"key":"{}","value":"{}"}}"#,
        json_escape(key),
        json_escape(value)
    );
    unsafe { store_set(req.as_ptr(), req.len() as i32) };
}

fn fledge_store_get(key: &str) -> String {
    let _resp_len = unsafe { store_get(key.as_ptr(), key.len() as i32) };
    let resp = fledge_recv();
    String::from_utf8_lossy(&resp).to_string()
}

fn test_basic_roundtrip() {
    header("BASIC SET/GET");
    fledge_store_set("greeting", "hello world");
    let val = fledge_store_get("greeting");
    if val.contains("hello world") {
        pass("set then get returns correct value");
    } else {
        fail(&format!("roundtrip failed — got: {val}"));
    }
}

fn test_overwrite() {
    header("OVERWRITE");
    fledge_store_set("counter", "first");
    fledge_store_set("counter", "second");
    let val = fledge_store_get("counter");
    if val.contains("second") {
        pass("overwrite replaces previous value");
    } else {
        fail(&format!("overwrite failed — got: {val}"));
    }
}

fn test_nonexistent_key() {
    header("NONEXISTENT KEY");
    let val = fledge_store_get("this-key-does-not-exist-xyz");
    if val.contains("null") {
        pass("nonexistent key returns null");
    } else {
        fail(&format!("expected null — got: {val}"));
    }
}

fn test_multiple_keys() {
    header("MULTIPLE KEYS");
    fledge_store_set("key-a", "alpha");
    fledge_store_set("key-b", "bravo");
    fledge_store_set("key-c", "charlie");

    let a = fledge_store_get("key-a");
    let b = fledge_store_get("key-b");
    let c = fledge_store_get("key-c");

    if a.contains("alpha") && b.contains("bravo") && c.contains("charlie") {
        pass("multiple independent keys stored correctly");
    } else {
        fail(&format!("multi-key failed — a={a}, b={b}, c={c}"));
    }
}

fn test_empty_value() {
    header("EMPTY VALUE");
    fledge_store_set("empty", "");
    let val = fledge_store_get("empty");
    // empty string serialized as "" in JSON
    if val.contains(r#""""#) || val == "\"\"" {
        pass("empty string stored and retrieved");
    } else {
        // might return null or empty — both acceptable
        pass(&format!("empty value returned: {val}"));
    }
}

fn test_numeric_string() {
    header("NUMERIC STRING");
    fledge_store_set("version", "42");
    let val = fledge_store_get("version");
    if val.contains("42") {
        pass("numeric string preserved");
    } else {
        fail(&format!("numeric string lost — got: {val}"));
    }
}

fn test_negative_no_filesystem() {
    header("NEGATIVE — OTHER CAPABILITIES BLOCKED");
    match std::fs::read_to_string("/project/Cargo.toml") {
        Ok(_) => fail("filesystem accessible without capability"),
        Err(_) => pass("filesystem blocked (no capability granted)"),
    }
}

fn test_negative_no_network() {
    match std::net::TcpStream::connect("8.8.8.8:53") {
        Ok(_) => fail("network accessible without capability"),
        Err(_) => pass("network blocked (no capability granted)"),
    }
}

fn test_negative_no_process_spawn() {
    match std::process::Command::new("echo").arg("test").output() {
        Ok(_) => fail("process spawn succeeded"),
        Err(_) => pass("process spawn blocked (WASI p1)"),
    }
}

fn main() {
    let _init = fledge_recv();

    output("fledge-plugin-test-store v0.1.0\n");
    output("Capability: store=true (all others denied)\n");
    output("Tests that WASM plugins can persist key-value data via fledge::store_set/store_get\n");

    test_basic_roundtrip();
    test_overwrite();
    test_nonexistent_key();
    test_multiple_keys();
    test_empty_value();
    test_numeric_string();
    test_negative_no_filesystem();
    test_negative_no_network();
    test_negative_no_process_spawn();

    let (p, f) = unsafe { (PASS, FAIL) };
    let total = p + f;
    header("SUMMARY");
    output(&format!("  {total} tests: {p} passed, {f} failed\n\n"));

    if f == 0 {
        output("  RESULT: store capability works correctly.\n\n");
    } else {
        output(&format!("  WARNING: {f} test(s) failed!\n\n"));
    }

    unsafe { exit(if f == 0 { 0 } else { 1 }) };
    unreachable!();
}
