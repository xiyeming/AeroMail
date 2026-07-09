/**
 * Decodes Gmail's modified UTF-7 encoding used for IMAP folder names.
 * RFC 3501 defines a modified UTF-7 for IMAP mailbox names where:
 * - `&` starts an encoded sequence
 * - `-` ends an encoded sequence
 * - Characters between are modified Base64 encoded UTF-16BE
 */
export function decodeModifiedUtf7(str: string): string {
  if (!str.includes('&')) return str;
  return str.replace(/&([A-Za-z0-9+/,]*)-/g, (_, encoded: string) => {
    if (encoded === '') return '&';
    // Convert modified Base64 (uses , instead of /) to standard Base64
    const base64 = encoded.replace(/,/g, '/');
    // Decode Base64 to bytes
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    // Decode UTF-16BE to string
    const decoder = new TextDecoder('utf-16be');
    return decoder.decode(bytes);
  });
}
