// crates/azadi-macros/src/evaluator/output.rs

/// Indicates how a piece of output relates to the original source.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SpanKind {
    /// Literal text from the source document or a textual block.
    Literal,
    /// Text substituted from expanding a macro body.
    MacroBody {
        macro_name: String,
    },
    /// Text substituted from an argument value at a macro call site.
    MacroArg {
        macro_name: String,
        param_name: String,
    },
    /// Text substituted from a global setting or without macro context.
    VarBinding {
        var_name: String,
    },
    /// Text generated programmatically (e.g. Rhai/Python script results, builtins)
    /// that has no direct corresponding source token for its content.
    Computed,
}

/// Byte-offset span referencing the source token that produced a piece of output.
///
/// Fields mirror `Token.src`, `Token.pos`, `Token.length` — no conversion needed.
/// Line/col can be derived on demand by scanning `source[..pos]` for `\n`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceSpan {
    /// Source file index (same as `Token.src`).
    pub src: u32,
    /// Byte offset in the source string (same as `Token.pos`).
    pub pos: usize,
    /// Byte length of the span (same as `Token.length`).
    pub length: usize,
    /// The kind of expansion that produced this text.
    pub kind: SpanKind,
}

/// Generic output sink for the evaluator.
///
/// The evaluator calls `push_str` for every piece of text it produces,
/// providing the `SourceSpan` of the token that generated it.
/// `push_untracked` is used for text whose origin cannot be attributed to
/// a single source span (e.g. Rhai/Python script results).
pub trait EvalOutput {
    /// Append `text` that originated at `span` in the source.
    fn push_str(&mut self, text: &str, span: SourceSpan);

    /// Append text with no span information (computed/script results).
    fn push_untracked(&mut self, text: &str);

    /// Consume the accumulator and return the rendered string.
    fn finish(self) -> String;
}

/// Fast-path output accumulator — ignores span info, just collects text.
///
/// This is functionally identical to the existing `String`-based output in
/// `Evaluator::evaluate()`.  Zero overhead: span arguments are discarded.
#[derive(Debug)]
pub struct PlainOutput {
    buf: String,
}

impl PlainOutput {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
        }
    }
}

impl Default for PlainOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalOutput for PlainOutput {
    #[inline]
    fn push_str(&mut self, text: &str, _span: SourceSpan) {
        self.buf.push_str(text);
    }

    #[inline]
    fn push_untracked(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    fn finish(self) -> String {
        self.buf
    }
}

/// A record of a contiguous sequence of bytes in the generated output and
/// the source span that produced it.
#[derive(Debug, Clone)]
pub struct SpanEntry {
    /// Byte offset in the output buffer where this span begins.
    pub out_offset: usize,
    /// Byte length of this span in the output buffer.
    pub out_len: usize,
    /// The source span that generated this text.
    pub span: SourceSpan,
}

/// Output accumulator that records precise provenance for every emitted byte.
///
/// The `spans` vector can be saved to the database and used by `azadi backprop`
/// to trace generated output lines back to their origin in the macro arguments
/// or macro bodies.
#[derive(Debug)]
pub struct TracingOutput {
    buf: String,
    pub spans: Vec<SpanEntry>,
}

impl TracingOutput {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            spans: Vec::new(),
        }
    }
}

impl Default for TracingOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalOutput for TracingOutput {
    fn push_str(&mut self, text: &str, span: SourceSpan) {
        if text.is_empty() {
            return;
        }
        self.spans.push(SpanEntry {
            out_offset: self.buf.len(),
            out_len: text.len(),
            span,
        });
        self.buf.push_str(text);
    }

    fn push_untracked(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        // For untracked text, we emit a span with kind=Computed and 0-length source.
        self.spans.push(SpanEntry {
            out_offset: self.buf.len(),
            out_len: text.len(),
            span: SourceSpan {
                src: 0,
                pos: 0,
                length: 0,
                kind: SpanKind::Computed,
            },
        });
        self.buf.push_str(text);
    }

    fn finish(self) -> String {
        self.buf
    }
}

/// A serialized entry stored in the `macro_map` database table.
/// It maps an output line (indirectly via the table key) to the original
/// `.md` source file that generated it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MacroMapEntry {
    /// Path of the source (literate) file containing the original text.
    pub src_file: String,
    /// 0-indexed line number within the source file.
    pub src_line: u32,
    /// 0-indexed column (byte offset) within the source line.
    pub src_col: u32,
    /// The kind of macro expansion that produced this text.
    pub kind: SpanKind,
}

use crate::evaluator::state::SourceManager;

impl TracingOutput {
    /// Convert the flat byte-level span entries into line-by-line `MacroMapEntry`s
    /// suitable for storage in the `macro_map` database table.
    /// 
    /// Returns a list of `(expanded_line_index, MacroMapEntry)`.
    pub fn into_macro_map_entries(
        &self,
        sources: &SourceManager,
    ) -> Vec<(u32, MacroMapEntry)> {
        let mut results = Vec::new();
        
        let mut line_start_offset = 0;
        let mut out_line_idx = 0;
        let mut span_iter = self.spans.iter().peekable();
        
        while line_start_offset < self.buf.len() {
            let line_end_offset = self.buf[line_start_offset..]
                .find('\n')
                .map(|idx| line_start_offset + idx)
                .unwrap_or(self.buf.len());
            
            // Advance our span iterator to find the span covering this line's start
            let mut active_span = None;
            while let Some(span) = span_iter.peek() {
                if span.out_offset + span.out_len <= line_start_offset {
                    span_iter.next();
                    continue;
                }
                if span.out_offset > line_end_offset {
                    break;
                }
                active_span = Some((*span).clone());
                break;
            }
            
            if let Some(entry) = active_span {
                if let Some(src_path) = sources.source_files().get(entry.span.src as usize) {
                    if let Some(src_content_bytes) = sources.get_source(entry.span.src) {
                        let src_content = String::from_utf8_lossy(src_content_bytes);
                        let (src_line, src_col) = find_line_col_0_indexed(&src_content, entry.span.pos);
                        
                        results.push((
                            out_line_idx,
                            MacroMapEntry {
                                src_file: src_path.to_string_lossy().into_owned(),
                                src_line,
                                src_col,
                                kind: entry.span.kind.clone(),
                            }
                        ));
                    }
                }
            }
            
            line_start_offset = line_end_offset + 1;
            out_line_idx += 1;
        }
        
        results
    }
}

/// Helper to convert a byte offset into a 0-indexed (line, col)
fn find_line_col_0_indexed(text: &str, byte_offset: usize) -> (u32, u32) {
    let offset = byte_offset.min(text.len());
    let prefix = &text[..offset];
    let newlines = prefix.bytes().filter(|&b| b == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    // col is char-indexed usually, but we'll stick to byte offset within line for now
    let col = (offset - line_start) as u32;
    (newlines, col)
}
