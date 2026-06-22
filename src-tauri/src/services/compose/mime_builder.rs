use lettre::Message;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, MultiPart, SinglePart};

use crate::error::AeroError;
use crate::models::compose::ComposeDraft;

#[cfg(test)]
use crate::models::compose::AttachmentDraft;

/// Builds an RFC-compliant MIME message from a draft.
pub fn build_message(
    draft: &ComposeDraft,
    from_address: &str,
    from_name: &str,
    attachment_bytes: &[(String, Vec<u8>)], // (attachment_id, bytes)
) -> Result<Vec<u8>, AeroError> {
    let from_header = format!("{from_name} <{from_address}>")
        .parse()
        .map_err(|e| AeroError::InvalidRecipient(format!("invalid from: {e}")))?;
    let mut builder = Message::builder().from(from_header);

    // Set In-Reply-To when replying to a mail with a known Message-ID
    if let Some(ref reply_context) = draft.reply_context {
        if let Some(ref original_message_id) = reply_context.original_message_id {
            let id = original_message_id.trim();
            builder = builder.in_reply_to(id.to_string());
        }
    }

    for addr in &draft.to {
        let to_header = addr
            .parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid to {addr}: {e}")))?;
        builder = builder.to(to_header);
    }
    for addr in &draft.cc {
        let cc_header = addr
            .parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid cc {addr}: {e}")))?;
        builder = builder.cc(cc_header);
    }
    for addr in &draft.bcc {
        let bcc_header = addr
            .parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid bcc {addr}: {e}")))?;
        builder = builder.bcc(bcc_header);
    }

    let message = builder
        .subject(draft.subject.clone())
        .multipart(build_body_and_attachments(draft, attachment_bytes)?)
        .map_err(|e| AeroError::MailBuilderFailed(e.to_string()))?;

    Ok(message.formatted())
}

fn content_type(mime: &str) -> ContentType {
    ContentType::parse(mime).unwrap_or_else(|_| {
        ContentType::parse("application/octet-stream")
            .unwrap_or_else(|_| unreachable!("static fallback content type"))
    })
}

fn build_body_and_attachments(
    draft: &ComposeDraft,
    attachment_bytes: &[(String, Vec<u8>)],
) -> Result<MultiPart, AeroError> {
    // Build alternative body (text + html)
    let body_part = MultiPart::alternative()
        .singlepart(
            SinglePart::builder()
                .header(content_type("text/plain; charset=utf-8"))
                .body(draft.body_text.clone()),
        )
        .singlepart(
            SinglePart::builder()
                .header(content_type("text/html; charset=utf-8"))
                .body(draft.body_html.clone()),
        );

    if draft.attachments.is_empty() {
        return Ok(body_part);
    }

    // Build mixed part: body + attachments
    let mut mixed = MultiPart::mixed().multipart(body_part);

    for attachment in &draft.attachments {
        let bytes = attachment_bytes
            .iter()
            .find(|(id, _)| id == &attachment.id)
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| AeroError::AttachmentNotFound(attachment.id.clone()))?;

        let content_type = content_type(&attachment.mime_type);

        if attachment.is_inline {
            let content_id = attachment
                .content_id
                .clone()
                .unwrap_or_else(|| attachment.id.clone());
            let part = Attachment::new_inline_with_name(content_id, attachment.filename.clone())
                .body(bytes, content_type);
            mixed = mixed.singlepart(part);
        } else {
            let part = Attachment::new(attachment.filename.clone()).body(bytes, content_type);
            mixed = mixed.singlepart(part);
        }
    }

    Ok(mixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_draft() -> ComposeDraft {
        ComposeDraft {
            id: "d1".to_string(),
            account_id: "a1".to_string(),
            reply_context: None,
            subject: "Hello".to_string(),
            to: vec!["to@example.com".to_string()],
            cc: vec![],
            bcc: vec![],
            body_html: "<p>Hello</p>".to_string(),
            body_text: "Hello".to_string(),
            attachments: vec![],
            saved_at: 0,
            synced_at: None,
            remote_uid: None,
        }
    }

    #[test]
    fn builds_text_html_message() {
        let draft = sample_draft();
        let bytes = build_message(&draft, "from@example.com", "Sender", &[]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("to@example.com"));
    }

    #[test]
    fn builds_message_with_attachment() {
        let mut draft = sample_draft();
        draft.attachments.push(AttachmentDraft {
            id: "att1".to_string(),
            filename: "note.txt".to_string(),
            mime_type: "text/plain".to_string(),
            size: 5,
            local_path: None,
            content_id: None,
            is_inline: false,
            preview_url: None,
        });
        let bytes = build_message(
            &draft,
            "from@example.com",
            "Sender",
            &[("att1".to_string(), b"hello".to_vec())],
        )
        .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("note.txt"));
        assert!(text.contains("hello"));
    }

    #[test]
    fn builds_reply_with_in_reply_to() {
        let mut draft = sample_draft();
        draft.reply_context = Some(crate::models::compose::ReplyContext {
            original_mail_id: "m1".to_string(),
            original_message_id: Some("<abc@example.com>".to_string()),
            kind: crate::models::compose::ReplyKind::Reply,
        });
        let bytes = build_message(&draft, "from@example.com", "Sender", &[]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("In-Reply-To: <abc@example.com>"));
    }
}
