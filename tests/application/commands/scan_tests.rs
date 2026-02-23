use super::*;

#[test]
fn extract_done_items_parses_list_and_stops_on_blank_line() {
    let message = "feat: improve parser\n\nDone:\n- first item\n* second item\n\nNotes:\n- not included";

    let items = extract_done_items(message, "done:");

    assert_eq!(items, vec!["first item", "second item"]);
}

#[test]
fn extract_done_items_stops_on_next_header() {
    let message = "done:\n- first\nNext Section:\n- should not be included";

    let items = extract_done_items(message, "done:");

    assert_eq!(items, vec!["first"]);
}

#[test]
fn parse_commits_includes_only_records_with_done_items() {
    let input = concat!(
        "abc123\x1f1700000000\x1fsubject\n\ndone:\n- ship feature\n\x1e",
        "def456\x1f1700000050\x1fsubject without done marker\x1e"
    );

    let commits = parse_commits(input, "done:");

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].hash, "abc123");
    assert_eq!(commits[0].done_items, vec!["ship feature"]);
    assert_eq!(commits[0].date.timestamp(), 1700000000);
}

#[test]
fn parse_commits_uses_epoch_for_invalid_timestamp() {
    let input = "abc123\x1fnot-a-number\x1fsubject\n\ndone:\n- keep\n\x1e";

    let commits = parse_commits(input, "done:");

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].date.timestamp(), 0);
}
