// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_messages::mention_extractor::extract_mentions_from_markdown;
use fluxer_messages::types::ExtractedMentionsResponse;

#[test]
fn markdown_extractor_matches_message_response_mention_rules() {
    let mentions = extract_mentions_from_markdown(Some(
        "ping <@101> <@!202> <@&303> <#404>\n`<@505>`\n```txt\n<#606>\n```",
    ));
    assert!(mentions.users.contains(&101));
    assert!(mentions.users.contains(&202));
    assert!(mentions.roles.contains(&303));
    assert!(mentions.channels.contains(&404));
    assert!(!mentions.users.contains(&505));
    assert!(!mentions.channels.contains(&606));
}

#[test]
fn markdown_extractor_keeps_link_text_mentions_but_not_url_mentions() {
    let mentions =
        extract_mentions_from_markdown(Some("[<@101>](https://example.com/<@202>) <#303>"));
    assert!(mentions.users.contains(&101));
    assert!(!mentions.users.contains(&202));
    assert!(mentions.channels.contains(&303));
}

#[test]
fn extracted_mentions_response_serializes_ids_as_strings() {
    let mentions = extract_mentions_from_markdown(Some("<@123456789012345678> <@&999> <#888>"));
    let response = ExtractedMentionsResponse::from(mentions);
    assert!(response.users.contains(&"123456789012345678".to_owned()));
    assert!(response.roles.contains(&"999".to_owned()));
    assert!(response.channels.contains(&"888".to_owned()));

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"123456789012345678\""));
    assert!(!json.contains("123456789012345678,"));
}

#[test]
fn everyone_and_here_survive_response_conversion() {
    let mentions = extract_mentions_from_markdown(Some("@everyone @here <@1>"));
    assert!(mentions.everyone);
    assert!(mentions.here);
    let response = ExtractedMentionsResponse::from(mentions);
    assert!(response.everyone);
    assert!(response.here);
}

#[test]
fn batch_extraction_matches_individual() {
    let contents = [
        "<@1> <@&2>".to_owned(),
        "`<@3>` <#4>".to_owned(),
        "@everyone".to_owned(),
    ];
    let batch: Vec<_> = contents
        .iter()
        .map(|c| extract_mentions_from_markdown(Some(c)))
        .collect();
    assert!(batch[0].users.contains(&1));
    assert!(batch[0].roles.contains(&2));
    assert!(!batch[1].users.contains(&3));
    assert!(batch[1].channels.contains(&4));
    assert!(batch[2].everyone);
}

#[test]
fn stress_test_large_message_batch() {
    let messages: Vec<String> = (0..500)
        .map(|i| {
            format!(
                "msg {i} <@{}> <@!{}> <@&{}> <#{}> `<@{}>` @everyone",
                10_000 + i,
                20_000 + i,
                30_000 + i,
                40_000 + i,
                50_000 + i
            )
        })
        .collect();
    let results: Vec<_> = messages
        .iter()
        .map(|m| extract_mentions_from_markdown(Some(m)))
        .collect();
    assert_eq!(results.len(), 500);
    for (i, result) in results.iter().enumerate() {
        let i = i as i64;
        assert!(result.users.contains(&(10_000 + i)));
        assert!(result.users.contains(&(20_000 + i)));
        assert!(result.roles.contains(&(30_000 + i)));
        assert!(result.channels.contains(&(40_000 + i)));
        assert!(!result.users.contains(&(50_000 + i)));
        assert!(result.everyone);
    }
}
