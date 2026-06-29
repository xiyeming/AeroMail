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

/// Wraps HTML content in a full document with inline styles for email clients.
/// Email clients (Gmail, Outlook, etc.) strip `<style>` blocks and `<head>`,
/// so all styling must be inline. A minimal `<style>` block is included as a
/// progressive-enhancement fallback for clients that preserve it (Apple Mail,
/// Thunderbird), while inline attributes ensure baseline rendering everywhere.
fn wrap_html_for_email(html: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <style>
    /* Progressive enhancement: clients that keep <style> get better defaults */
    body {{ margin:0; padding:0; font-family:Arial,Helvetica,sans-serif; font-size:14px; color:#333; line-height:1.6; }}
    a {{ color:#2563eb; text-decoration:underline; }}
    img {{ max-width:100%; height:auto; }}
    table {{ border-collapse:collapse; }}
  </style>
</head>
<body style="margin:0;padding:0;font-family:Arial,Helvetica,sans-serif;font-size:14px;color:#333;line-height:1.6;">
{html}
</body>
</html>"#
    )
}

/// 为 Tiptap 生成的语义 HTML 注入内联样式，确保邮件客户端正确显示。
/// 邮件客户端（Gmail、Outlook 等）会剥离 `<style>` 块，所有样式必须内联。
/// Tiptap 的 `TextStyle` 扩展已为字体、字号、颜色生成 inline style，
/// 此函数补充结构元素（段落、列表、标题、引用、链接等）的默认样式。
fn inject_inline_styles(html: &str) -> String {
    let mut result = html.to_string();

    // --- 结构元素默认样式 ---
    // 使用 merge_style_defaults 确保 Tiptap 已有的 text-align 等 inline style
    // 不会丢失 margin、line-height 等基础排版属性。

    // 段落：Tiptap 可能已有 text-align style，需合并而非覆盖
    result = merge_style_defaults(&result, "<p", "margin:0.5em 0;line-height:1.6;");

    // 标题
    result = merge_style_defaults(
        &result,
        "<h1",
        "margin:0.67em 0;font-size:2em;font-weight:700;line-height:1.3;",
    );
    result = merge_style_defaults(
        &result,
        "<h2",
        "margin:0.5em 0;font-size:1.5em;font-weight:700;line-height:1.3;",
    );
    result = merge_style_defaults(
        &result,
        "<h3",
        "margin:0.5em 0;font-size:1.17em;font-weight:700;line-height:1.3;",
    );

    // 列表
    result = inject_style_if_missing(&result, "<ul", "margin:0.5em 0;padding-left:1.5em;");
    result = inject_style_if_missing(&result, "<ol", "margin:0.5em 0;padding-left:1.5em;");
    result = inject_style_if_missing(&result, "<li", "margin:0.25em 0;");

    // 引用块
    result = inject_style_if_missing(
        &result,
        "<blockquote",
        "margin:0.5em 0;padding-left:1em;border-left:3px solid #ccc;color:#666;",
    );

    // 链接
    result = inject_style_if_missing(&result, "<a", "color:#2563eb;text-decoration:underline;");

    // 水平线
    result = inject_style_if_missing(
        &result,
        "<hr",
        "border:none;border-top:1px solid #ccc;margin:1em 0;",
    );

    // --- 表格样式 ---
    result = inject_style_if_missing(
        &result,
        "<table",
        "border-collapse:collapse;border-spacing:0;max-width:100%;",
    );
    result = inject_style_if_missing(
        &result,
        "<th",
        "border:1px solid #ccc;padding:6px 8px;text-align:left;background:#f5f5f5;font-weight:600;",
    );
    result = inject_style_if_missing(
        &result,
        "<td",
        "border:1px solid #ccc;padding:6px 8px;text-align:left;vertical-align:top;",
    );

    // --- 代码样式 ---
    result = inject_style_if_missing(
        &result,
        "<pre",
        "margin:0.5em 0;padding:0.75em 1em;background:#f6f8fa;border-radius:6px;overflow-x:auto;white-space:pre;font-size:0.9em;line-height:1.45;",
    );
    result = inject_style_if_missing(
        &result,
        "<code",
        "font-family:'SFMono-Regular',Consolas,'Liberation Mono',Menlo,monospace;font-size:0.9em;",
    );

    result
}

/// 如果 HTML 标签没有 style 属性，则注入指定的内联样式。
fn inject_style_if_missing(html: &str, tag: &str, style: &str) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(html.len() + 256);
    let tag_lower = tag.to_lowercase();
    let mut pos = 0;

    while let Some(offset) = rest_lower(html, pos).find(&tag_lower) {
        let abs_pos = pos + offset;
        result.push_str(&html[pos..abs_pos]);

        // 找到标签的结束位置（> 或 />）
        let tag_content_start = abs_pos + tag.len();
        if let Some(gt_offset) = html[tag_content_start..].find('>') {
            let tag_end = tag_content_start + gt_offset;
            let tag_content = &html[tag_content_start..tag_end];

            if tag_content.to_lowercase().contains("style=") {
                // 已有 style 属性，保留原样
                result.push_str(&html[abs_pos..=tag_end]);
            } else {
                // 没有 style 属性，注入
                result.push_str(tag);
                let _ = write!(result, " style=\"{style}\"");
                result.push_str(&html[tag_content_start..=tag_end]);
            }
            pos = tag_end + 1;
        } else {
            // 没有找到 >，保留原样
            result.push_str(&html[abs_pos..]);
            pos = html.len();
            break;
        }
    }

    result.push_str(&html[pos..]);
    result
}

/// 合并默认样式到已有 style 属性中。与 `inject_style_if_missing` 不同，
/// 此函数不会跳过已有 style 的元素，而是将默认样式中缺失的 CSS 属性
/// 追加到已有 style 的末尾，确保 Tiptap 生成的 text-align 等样式不丢失
/// margin、line-height 等基础排版属性。
fn merge_style_defaults(html: &str, tag: &str, defaults: &str) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(html.len() + 256);
    let tag_lower = tag.to_lowercase();
    let mut pos = 0;

    // 解析默认样式为 (property, value) 列表
    let default_props: Vec<(&str, &str)> = defaults
        .split(';')
        .filter_map(|decl| {
            let decl = decl.trim();
            if decl.is_empty() {
                return None;
            }
            let (prop, val) = decl.split_once(':')?;
            Some((prop.trim(), val.trim()))
        })
        .collect();

    while let Some(offset) = rest_lower(html, pos).find(&tag_lower) {
        let abs_pos = pos + offset;
        result.push_str(&html[pos..abs_pos]);

        let tag_content_start = abs_pos + tag.len();
        if let Some(gt_offset) = html[tag_content_start..].find('>') {
            let tag_end = tag_content_start + gt_offset;
            let tag_content = &html[tag_content_start..tag_end];

            if tag_content.to_lowercase().contains("style=") {
                // 已有 style：提取现有值，追加缺失的默认属性
                let tag_content_lower = tag_content.to_lowercase();
                let Some(style_start) = tag_content_lower.find("style=") else {
                    continue;
                };
                let after_style = &tag_content[style_start + 6..];
                let quote = after_style.chars().next().unwrap_or('"');
                let after_quote = &after_style[1..];
                if let Some(style_end) = after_quote.find(quote) {
                    let existing_style = &after_quote[..style_end];
                    let before_style = &tag_content[..style_start];
                    let after_close = &after_quote[style_end + 1..];

                    // 检查哪些默认属性在现有 style 中缺失
                    let mut missing: Vec<String> = Vec::new();
                    for (prop, val) in &default_props {
                        let prop_lower = prop.to_lowercase();
                        let already_has = existing_style
                            .split(';')
                            .any(|d| d.trim().starts_with(&*prop_lower));
                        if !already_has {
                            missing.push(format!("{prop}:{val}"));
                        }
                    }

                    if missing.is_empty() {
                        // 没有缺失属性，保留原样
                        result.push_str(&html[abs_pos..=tag_end]);
                    } else {
                        // 追加缺失属性
                        result.push_str(tag);
                        result.push_str(before_style);
                        let _ = write!(result, "style=\"{existing_style}");
                        if !existing_style.trim().ends_with(';') {
                            result.push(';');
                        }
                        for m in &missing {
                            let _ = write!(result, "{m};");
                        }
                        result.push(quote);
                        result.push_str(after_close);
                    }
                } else {
                    result.push_str(&html[abs_pos..=tag_end]);
                }
            } else {
                // 没有 style 属性，直接注入全部默认样式
                result.push_str(tag);
                let _ = write!(result, " style=\"{defaults}\"");
                result.push_str(&html[tag_content_start..=tag_end]);
            }
            pos = tag_end + 1;
        } else {
            result.push_str(&html[abs_pos..]);
            pos = html.len();
            break;
        }
    }

    result.push_str(&html[pos..]);
    result
}

/// 辅助函数：返回从 pos 开始的子串（小写），用于大小写不敏感搜索。
fn rest_lower(s: &str, pos: usize) -> String {
    s[pos..].to_lowercase()
}

/// Strips HTML tags from a string, returning plain text.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace and trim
    result
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Converts `<img src="data:..." data-cid="...">` to `<img src="cid:...">` for email MIME.
fn convert_inline_images_to_cid(html: &str) -> String {
    // Replace data-cid images with cid: references
    let mut result = html.to_string();

    // Find all <img> tags with data-cid attribute and replace their src
    while let Some(start) = result.find("data-cid=\"") {
        // Find the opening <img tag before data-cid
        let before_data_cid = &result[..start];
        let tag_start = before_data_cid.rfind("<img").unwrap_or(0);

        // Find the closing > of the img tag
        let after_data_cid = &result[start + 10..];
        if let Some(tag_end_in_rest) = after_data_cid.find('>') {
            let tag_end = start + 10 + tag_end_in_rest + 1;
            let img_tag = &result[tag_start..tag_end];

            // Extract the cid value
            if let Some(cid_start) = img_tag.find("data-cid=\"") {
                let cid_value_start = cid_start + 10;
                if let Some(cid_end) = img_tag[cid_value_start..].find('"') {
                    let cid = &img_tag[cid_value_start..cid_value_start + cid_end];

                    // Replace the entire img tag: remove data-cid, change src to cid:
                    let new_tag = img_tag
                        .replace(&format!("data-cid=\"{cid}\""), "")
                        .replacen("src=\"data:", &format!("src=\"cid:{cid}\""), 1);

                    // But we need to handle the base64 src properly
                    // Find and replace the src attribute completely
                    if let Some(src_start) = new_tag.find("src=\"") {
                        let src_val_start = src_start + 5;
                        if let Some(src_end) = new_tag[src_val_start..].find('"') {
                            let old_src = &new_tag[src_val_start..src_val_start + src_end];
                            let fixed_tag = new_tag.replace(
                                &format!("src=\"{old_src}\""),
                                &format!("src=\"cid:{cid}\""),
                            );
                            result = format!(
                                "{}{}{}",
                                &result[..tag_start],
                                fixed_tag,
                                &result[tag_end..]
                            );
                            continue;
                        }
                    }

                    result = format!("{}{}{}", &result[..tag_start], new_tag, &result[tag_end..]);
                }
            }
        } else {
            break;
        }
    }

    result
}

fn build_body_and_attachments(
    draft: &ComposeDraft,
    attachment_bytes: &[(String, Vec<u8>)],
) -> Result<MultiPart, AeroError> {
    // Convert base64 inline images back to cid: references for email
    let html = convert_inline_images_to_cid(&draft.body_html);

    // 为语义 HTML 注入内联样式（表格边框、代码块背景等）
    let html = inject_inline_styles(&html);

    // Wrap HTML in a full document with inline styles for email clients
    let html = wrap_html_for_email(&html);

    // Ensure body_text is never empty (some clients show nothing if text/plain is empty)
    let text = if draft.body_text.trim().is_empty() {
        strip_html_tags(&html)
    } else {
        draft.body_text.clone()
    };

    // Build alternative body (text + html)
    let body_part = MultiPart::alternative()
        .singlepart(
            SinglePart::builder()
                .header(content_type("text/plain; charset=utf-8"))
                .body(text),
        )
        .singlepart(
            SinglePart::builder()
                .header(content_type("text/html; charset=utf-8"))
                .body(html),
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
    fn builds_text_html_message() -> Result<(), Box<dyn std::error::Error>> {
        let draft = sample_draft();
        let bytes = build_message(&draft, "from@example.com", "Sender", &[])?;
        let text = String::from_utf8(bytes)?;
        assert!(text.contains("Hello"));
        assert!(text.contains("to@example.com"));
        Ok(())
    }

    #[test]
    fn builds_message_with_attachment() -> Result<(), Box<dyn std::error::Error>> {
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
        )?;
        let text = String::from_utf8(bytes)?;
        assert!(text.contains("note.txt"));
        assert!(text.contains("hello"));
        Ok(())
    }

    #[test]
    fn builds_reply_with_in_reply_to() -> Result<(), Box<dyn std::error::Error>> {
        let mut draft = sample_draft();
        draft.reply_context = Some(crate::models::compose::ReplyContext {
            original_mail_id: "m1".to_string(),
            original_message_id: Some("<abc@example.com>".to_string()),
            kind: crate::models::compose::ReplyKind::Reply,
        });
        let bytes = build_message(&draft, "from@example.com", "Sender", &[])?;
        let text = String::from_utf8(bytes)?;
        assert!(text.contains("In-Reply-To: <abc@example.com>"));
        Ok(())
    }
}
