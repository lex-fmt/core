# Lex Output Quality Assurance Review

**Date:** November 20, 2025
**Reviewer:** Gemini CLI Agent
**Context:** Pre-launch review of Parser AST, HTML Export, and Markdown Export.

## 1. Executive Summary

The Lex parser demonstrates high robustness in handling the core structural philosophy of the language: "invisible structure" via indentation. The AST generation is accurate across simple and deeply nested documents.

* **Parsing/AST:** ✅ **Pass**. The line-grouping and semantic indentation logic correctly handles sessions, lists, definitions, and verbatim blocks.
* **HTML Export:** ✅ **Strong Pass**. The output is semantic, accessible, and visually styled with a modern default theme.
* **Markdown Export:** ⚠️ **Pass with Warnings**. While structurally sound, the export suffers from aggressive character escaping (breaking links/references) and "lossy" conversion of Lex-specific features like annotations.

## 2. Methodology

We analyzed the following benchmark files using the `lex-cli`:

1. `010-kitchensink.lex` (Feature coverage)
2. `20-ideas-naked.lex` (Standard document flow)
3. `30-a-place-for-ideas.lex` (Deep nesting)
4. `docs/dev/guides/on-all-of-lex.lex` (Complex documentation structure)

**Tools Used:**

* `ast-linetreeviz` & `ast-treeviz` for structural verification.
* `convert --to html` & `convert --to markdown` for interoperability checks.

## 3. Detailed Findings

### 3.1 Parsing & AST Structure

The AST correctly resolves the "Indentation Wall" concept.

* **Sessions:** Correctly identify headers vs. content. The parser handles "blank line" separators correctly, distinguishing between content belonging to a parent session vs. a child session.
* **Lists:** "Trifecta" tests (mixed lists, paragraphs, and sessions) show that the parser can disambiguate between a list item and a paragraph starting with a dash if the context implies it.
* **Verbatim Blocks:** The indentation logic for verbatim blocks (In-Flow vs. Full-Width) is correctly represented in the AST, preserving internal whitespace while dedenting the block correctly.

### 3.2 HTML Export

The HTML output is production-ready.

* **Semantics:** Uses `<section>` for sessions, `<dl>`/`<dt>`/`<dd>` for definitions, and `<pre>`/`<code>` for verbatim blocks.
* **Styling:** The default CSS (injected into `<head>`) is responsive and clean.
* **Annotations:** Lex annotations are converted to HTML comments (e.g., `<!-- lex:warning -->`) or rendered structurally if they wrap content.
* **References:**
  * Internal links: `<a href="#3.2">` work correctly.
  * Citations: `<a href="@spec2025...">` generates an anchor, though the `href` value might need post-processing to point to a real bibliography URL in a real-world deployment.

### 3.3 Markdown Export

The Markdown export is functional but has specific friction points regarding Lex's richer feature set.

* **Escaping Issues:** The exporter aggressively escapes brackets `[` and `]`, which breaks potential Markdown links if they weren't fully resolved.
  * *Example:* `\[@spec2025, pp. 45-46\]` vs `[@spec2025, pp. 45-46]`.
* **Annotations:** Converted to HTML comments (`<!-- lex:author -->`). This preserves the data but hides it from the rendered Markdown view. This is likely a necessary compromise but worth noting.
* **Math:** Math spans like `#$$E=mc^2$$#` are converted, but surrounding escaping logic should be double-checked to ensure LaTeX parsers (like MathJax) pick them up correctly in the target Markdown environment.
* **Nested Structures:** Deeply nested lists/definitions are flattened into standard Markdown indentation. This is generally correct, but Markdown implementations vary in how they handle "4-space" vs "2-space" indents for sub-lists.

## 4. Specific Issues & Recommendations

### Issue A: Reference Escaping in Markdown

**Severity:** Medium (RESOLVED)
**Observation:** References are rendered as `\[#3.2\]` in the Markdown output.
**Impact:** Most Markdown parsers will render this as literal text `[#3.2]` rather than a clickable link.
**Resolution:** Logic added to `lex-babel/src/formats/markdown/serializer.rs`. References starting with `http`, `#`, `/`, or `.` are converted to explicit Markdown links. Citations (`@`) link to `#ref-...`.

### Issue B: Citation `href` format in HTML

**Severity:** Low (Enhancement) (RESOLVED)
**Observation:** Citations render as `<a href="@john-2023">`.
**Impact:** Clicking this link usually results in a 404 or invalid protocol error in browsers unless intercepted by JS.
**Resolution:** Logic added to `lex-babel/src/formats/html/serializer.rs`. Citations now render as anchors with `#ref-` prefix (e.g., `href="#ref-john-2023"`).

### Issue C: Markdown List Formatting

**Severity:** Low (VERIFIED - NO ACTION NEEDED)
**Observation:** Lex supports very rich content inside lists. Markdown is fragile here.
**Recommendation:** Ensure that multi-paragraph list items in Markdown are consistently indented by 4 spaces (or 1 tab) relative to the bullet point to ensure they are treated as part of the list item and not a broken break.
**Verification:** The Markdown exporter uses Comrak, a mature CommonMark-compliant library that automatically handles proper indentation for multi-paragraph list items. Added comprehensive tests to verify list formatting behavior. No code changes needed - Comrak already implements correct CommonMark list formatting.

## 5. Conclusion

The Lex parser core is solid. The HTML exporter is excellent and ready for use. The Markdown exporter is a useful utility but requires minor tuning regarding character escaping and link generation to be truly "interoperable" with standard Markdown readers (GitHub, Obsidian, VS Code).
