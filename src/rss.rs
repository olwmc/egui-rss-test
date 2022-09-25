use std::fmt;
use ureq;

use feed_rs::{parser, model::Feed};

// Wrapping errors in an enum so they can bubble up
#[derive(Debug)]
pub enum RssError {
    RssParseError(feed_rs::parser::ParseFeedError),
    UreqError(ureq::Error)
}

impl std::error::Error for RssError {}

impl From<parser::ParseFeedError> for RssError {
    fn from(err: parser::ParseFeedError) -> Self {
        RssError::RssParseError(err)
    }
}

impl From<ureq::Error> for RssError {
    fn from(err: ureq::Error) -> Self {
        RssError::UreqError(err)
    }
}

impl fmt::Display for RssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RssError::RssParseError(err) => 
                write!(f, "{}", err),
            RssError::UreqError(err) => 
                write!(f, "{}", err),
        }
    }
}

pub fn get_articles(url: &str) -> Result<Feed, RssError> {
    // We're using unwrap here because if this is an error it should actually panic
    // becuase I need to figure out when into_string breaks. Technically this breaks
    // automatically if the size of the feed is over 10mb
    let response = ureq::get(url).call()?.into_string().unwrap();
    let feed = parser::parse(response.as_bytes())?;

    Ok(feed)
}
