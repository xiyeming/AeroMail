use mail_parser::{MessageParser, MimeHeaders};

use crate::error::AeroError;
use crate::models::mail::{ParsedAttachment, ParsedMail};

/// Parses a raw MIME email into a [`ParsedMail`] struct.
///
/// # Errors
///
/// Returns an error if the mail cannot be parsed.
pub fn parse_mail(raw: &[u8]) -> Result<ParsedMail, AeroError> {
    let message = MessageParser::new()
        .parse(raw)
        .ok_or_else(|| AeroError::Internal("Failed to parse MIME message".into()))?;

    let from = message.from();
    let (from_name, from_address) =
        from.and_then(|a| a.iter().next())
            .map_or((None, None), |addr| {
                (
                    addr.name().map(String::from),
                    addr.address().map(String::from),
                )
            });

    let to_addresses = format_address_list(message.to());
    let cc_addresses = format_address_list(message.cc());

    let date = message.date().map(mail_parser::DateTime::to_timestamp);

    let body_html = message.body_html(0).map(String::from);
    let body_text = message.body_text(0).map(String::from);

    let has_attachments = message.attachment_count() > 0;
    let attachments = parse_attachments(&message);

    // Convert IMAP flags to strings
    let flags: Vec<String> = Vec::new(); // Flags are tracked separately via IMAP

    let message_id = message.message_id().map(String::from);

    Ok(ParsedMail {
        subject: message.subject().map(String::from),
        from_name,
        from_address,
        to_addresses,
        cc_addresses,
        date,
        body_html,
        body_text,
        has_attachments,
        flags,
        message_id,
        attachments,
    })
}

/// Extracts attachments from a parsed message.
fn parse_attachments(message: &mail_parser::Message<'_>) -> Vec<ParsedAttachment> {
    let mut attachments = Vec::new();

    for part in message.attachments() {
        let filename = part.attachment_name().map(String::from);
        let mime_type = part.content_type().map_or_else(
            || "application/octet-stream".to_string(),
            |ct| {
                let subtype = ct.c_subtype.as_deref().unwrap_or("octet-stream");
                format!("{}/{subtype}", ct.c_type)
            },
        );
        let content_id = part.content_id().map(String::from);
        let is_inline = part
            .content_disposition()
            .is_some_and(|cd| cd.c_type.eq_ignore_ascii_case("inline"));
        let data = part.contents().to_vec();
        let size = data.len();

        attachments.push(ParsedAttachment {
            filename,
            mime_type,
            size,
            content_id,
            is_inline,
            data,
        });
    }

    attachments
}

/// Formats an address list into a JSON string representation.
fn format_address_list(addrs: Option<&mail_parser::Address<'_>>) -> Option<String> {
    let addrs = addrs?;
    let addr_list = addrs.iter();

    let formatted: Vec<String> = addr_list
        .filter_map(|addr| {
            let address = addr.address()?;
            let name = addr.name();
            name.map_or_else(
                || Some(address.to_string()),
                |name| Some(format!("{name} <{address}>")),
            )
        })
        .collect();

    if formatted.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&formatted).unwrap_or_default())
    }
}
