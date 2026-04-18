use n2b_core::scanners::package_json::scan_package_json;
use proptest::prelude::*;

proptest! {
    #[test]
    fn scanner_never_panics_on_arbitrary_string(s in "[\\x20-\\x7E\\n\\t]{0,2048}") {
        // The scanner must either parse successfully or return a clean error.
        // It must never panic.
        let _ = scan_package_json("package.json", &s);
    }

    #[test]
    fn scanner_never_panics_on_valid_json_shape(
        name in "[a-zA-Z0-9-]{1,40}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
        dep_count in 0usize..20
    ) {
        let deps: serde_json::Map<String, serde_json::Value> = (0..dep_count)
            .map(|i| (format!("dep-{i}"), serde_json::Value::String(format!("^1.{i}.0"))))
            .collect();
        let pkg = serde_json::json!({ "name": name, "version": version, "dependencies": deps });
        let s = serde_json::to_string_pretty(&pkg).unwrap();
        let _ = scan_package_json("package.json", &s);
    }
}
