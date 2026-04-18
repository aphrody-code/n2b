// Round-trip test: every JSON baseline we captured must deserialize cleanly
// into the types generated from `schema/v2.json`. If this ever fails, the
// schema has drifted from the emitted JSON and someone broke the contract.

#[cfg(test)]
mod tests {
    use n2b_core::schema::N2bReport;

    #[test]
    fn deserializes_rpb_dashboard_baseline() {
        let raw = include_str!("../../../tests/rpb-dashboard-baseline/scan.json");
        let parsed: N2bReport = serde_json::from_str(raw)
            .expect("rpb-dashboard baseline must deserialize into N2bReport");
        assert!(parsed.files_scanned >= 1);
        assert_eq!(parsed.tool, "node2bun");
    }

    #[test]
    fn deserializes_fixture_baseline() {
        let raw = include_str!("../../../tests/snapshots/baseline/fixture.json");
        let _parsed: N2bReport =
            serde_json::from_str(raw).expect("fixture baseline must deserialize into N2bReport");
    }
}
