/**
 * Finds every balanced {{templateName ...}} block in raw wikitext, matching
 * MediaWiki template-invocation syntax: the name must be followed by whitespace,
 * "|", or "}" (not by more name characters — e.g. "Relic Set" must not match
 * inside "Relic + Set"). Balances nested {{ }} pairs so a template value that
 * itself contains a nested template (e.g. {{augment}}) doesn't truncate the block
 * at that inner "}}".
 */
export function findTemplateBlocks(wikitext: string, templateName: string): string[] {
  const blocks: string[] = [];
  const marker = `{{${templateName}`;
  let searchFrom = 0;

  while (true) {
    const start = wikitext.indexOf(marker, searchFrom);
    if (start === -1) break;

    const afterMarker = wikitext[start + marker.length];
    if (afterMarker !== undefined && !/[\s|}]/.test(afterMarker)) {
      searchFrom = start + marker.length;
      continue;
    }

    let depth = 0;
    let i = start;
    while (i < wikitext.length) {
      if (wikitext.startsWith("{{", i)) {
        depth++;
        i += 2;
      } else if (wikitext.startsWith("}}", i)) {
        depth--;
        i += 2;
        if (depth === 0) break;
      } else {
        i++;
      }
    }

    if (depth === 0) {
      blocks.push(wikitext.slice(start, i));
    }
    searchFrom = i || start + marker.length;
  }

  return blocks;
}

/**
 * Parses a template block's "|key=value" lines into a plain object. Assumes one
 * field per physical line — confirmed true of every AF3/Empyrean/Relic field this
 * tool reads (multi-line-looking values use literal "<br />" markup, not real
 * newlines, in BG-Wiki's actual wikitext).
 */
export function parseTemplateFields(block: string): Record<string, string> {
  const fields: Record<string, string> = {};

  for (const line of block.split("\n")) {
    const trimmed = line.trimStart();
    if (!trimmed.startsWith("|")) continue;

    const eqIndex = trimmed.indexOf("=");
    if (eqIndex === -1) continue;

    const key = trimmed.slice(1, eqIndex).trim();
    const value = trimmed.slice(eqIndex + 1).trim();
    fields[key] = value;
  }

  return fields;
}

/**
 * Strips MediaWiki wikilink syntax from a field value: "[[Target]]" -> "Target",
 * "[[Target|Display text]]" -> "Display text". Confirmed live that some AF3 job
 * fields use jobs=[[Scholar]] while most use jobs=Scholar — inconsistent wiki
 * authoring across pages, not a parsing bug. Returns the input unchanged if it
 * isn't wrapped as a wikilink.
 */
export function stripWikiLink(value: string): string {
  const match = value.match(/^\[\[([^\]|]+)(?:\|([^\]]+))?\]\]$/);
  if (!match) return value;
  return match[2] ?? match[1];
}
