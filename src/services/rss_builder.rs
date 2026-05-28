use rss::{ChannelBuilder, GuidBuilder, ItemBuilder};

use crate::domain::{extracted_item::RssItemData, feed::Feed};

pub fn build_rss(feed: &Feed, items: &[RssItemData]) -> String {
    let rss_items = items
        .iter()
        .map(|item| {
            ItemBuilder::default()
                .title(Some(item.title.clone()))
                .link(Some(item.link.clone()))
                .guid(Some(
                    GuidBuilder::default()
                        .value(item.link.clone())
                        .permalink(true)
                        .build(),
                ))
                .pub_date(Some(item.pub_date.to_rfc2822()))
                .description(item.description.clone())
                .build()
        })
        .collect::<Vec<_>>();

    ChannelBuilder::default()
        .title(feed.name.clone())
        .link(feed.site_url.clone())
        .description(
            feed.feed_description
                .clone()
                .unwrap_or_else(|| feed.name.clone()),
        )
        .items(rss_items)
        .build()
        .to_string()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn builds_rss_xml() {
        let feed = Feed {
            id: 1,
            name: "Example".to_string(),
            site_url: "https://example.com".to_string(),
            feed_description: Some("Desc".to_string()),
            selector_type: "css".to_string(),
            item_selector: "article".to_string(),
            title_selector: "h2".to_string(),
            link_selector: None,
            date_selector: None,
            content_selector: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let xml = build_rss(
            &feed,
            &[RssItemData {
                title: "Post".to_string(),
                link: "https://example.com/post".to_string(),
                pub_date: Utc::now(),
                description: Some("Body".to_string()),
            }],
        );
        assert!(xml.contains("<title>Example</title>"));
        assert!(xml.contains("<title>Post</title>"));
        assert!(xml.contains("<guid"));
        assert!(xml.contains("https://example.com/post"));
        assert!(xml.contains("Body"));
    }
}
