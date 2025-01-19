// <[@file src/noweb.rs]>=
// src/noweb.rs
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path};
use std::rc::Rc;

use crate::safe_writer::SafeFileWriter;
use crate::AzadiError;

/// Indicates file + line for error reporting.
#[derive(Debug, Clone)]
pub struct ChunkLocation {
    pub file_idx: usize,
    pub line: usize,
}

/// Represents a single definition of a named chunk.
#[derive(Debug, Clone)]
struct ChunkDef {
    content: Vec<String>,
    base_indent: usize,
    file_idx: usize,
    line: usize,
}

impl ChunkDef {
    fn new(base_indent: usize, file_idx: usize, line: usize) -> Self {
        Self {
            content: Vec::new(),
            base_indent,
            file_idx,
            line,
        }
    }
}

/// Each named chunk can have multiple definitions plus a reference counter.
#[derive(Debug)]
struct NamedChunk {
    definitions: Vec<ChunkDef>,
    references: usize,
}

impl NamedChunk {
    fn new() -> Self {
        Self {
            definitions: Vec::new(),
            references: 0,
        }
    }
}

/// Main store: chunk name -> Rc<RefCell<NamedChunk>>,
/// plus a list of which chunk names start with @file .
pub struct ChunkStore {
    chunks: HashMap<String, Rc<RefCell<NamedChunk>>>,
    file_chunks: Vec<String>,

    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,

    /// All file names for error reporting, indexed by file_idx.
    file_names: Vec<String>,
}

/// Check if the given path is safe (not absolute, no .., no colon).
fn path_is_safe(path: &str) -> Result<(), AzadiError> {
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(AzadiError::SecurityViolation(
            "Absolute paths are not allowed".to_string(),
        ));
    }
    if p.to_string_lossy().contains(':') {
        return Err(AzadiError::SecurityViolation(
            "Windows-style paths are not allowed".to_string(),
        ));
    }
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(AzadiError::SecurityViolation(
            "Path traversal is not allowed".to_string(),
        ));
    }
    Ok(())
}

impl ChunkStore {
    pub fn new(
        open_delim: &str,           // e.g. "<<"
        close_delim: &str,          // e.g. ">>"
        chunk_end: &str,            // e.g. "@"
        comment_markers: &[String], // e.g. ["#", "//"]
    ) -> Self {
        let od = regex::escape(open_delim);
        let cd = regex::escape(close_delim);

        // Build patterns that match lines like:
        //   # <<@replace @file chunk>>=
        //   # <<chunk>>=
        // for references:
        //   # <<chunk>>
        //   # <<@reversed chunk>>
        // for closings:
        //   # @
        let escaped_comments = comment_markers
            .iter()
            .map(|m| regex::escape(m))
            .collect::<Vec<_>>()
            .join("|");

        // Opening lines
        let open_pattern = format!(
            r"^(\s*)(?:{})?[ \t]*{}(?:@replace[ \t]+)?(?:@file[ \t]+)?([^\s]+){}=",
            escaped_comments, od, cd
        );
        // Reference lines
        let slot_pattern = format!(
            r"^(\s*)(?:{})?\s*{}(?:@file\s+|@reversed\s+)?([^\s>]+){}\s*$",
            escaped_comments, od, cd
        );
        // Closing lines
        let close_pattern = format!(
            r"^(?:{})?[ \t]*{}\s*$",
            escaped_comments,
            regex::escape(chunk_end)
        );

        Self {
            chunks: HashMap::new(),
            file_chunks: Vec::new(),
            open_re: Regex::new(&open_pattern).expect("Invalid open pattern"),
            slot_re: Regex::new(&slot_pattern).expect("Invalid slot pattern"),
            close_re: Regex::new(&close_pattern).expect("Invalid close pattern"),
            file_names: Vec::new(),
        }
    }

    pub fn add_file_name(&mut self, fname: &str) -> usize {
        let idx = self.file_names.len();
        self.file_names.push(fname.to_string());
        idx
    }

    fn validate_chunk_name(&self, chunk_name: &str, line: &str) -> bool {
        if line.contains("@file") {
            // Then chunk_name is a path
            path_is_safe(chunk_name).is_ok()
        } else {
            !chunk_name.is_empty() && !chunk_name.contains(char::is_whitespace)
        }
    }

    /// The main function for reading lines from the input text.
    /// - If the line opens a chunk, we define it (or replace it).
    /// - If the line closes a chunk, we end the current one.
    /// - Otherwise, if we’re inside a chunk, we add lines to it.
    /// Then we fill out file_chunks for any chunk name that starts with @file .
    pub fn read(&mut self, text: &str, file_idx: usize) {
        let mut current_chunk: Option<(String, usize)> = None;
        let mut line_no: i32 = -1;

        for line in text.lines() {
            line_no += 1;

            // Check if it's an opening line for a chunk
            if let Some(caps) = self.open_re.captures(line) {
                let indentation = caps.get(1).map_or("", |m| m.as_str());
                let base_name = caps.get(2).map_or("", |m| m.as_str()).to_string();

                let is_replace = line.contains("@replace");
                let is_file = line.contains("@file");
                // If line has @file, chunk name should be "@file something"
                let full_name = if is_file {
                    format!("@file {}", base_name)
                } else {
                    base_name
                };

                if self.validate_chunk_name(&full_name, line) {
                    // If this is a file chunk, check for existing definitions
                    // unless @replace is present
                    if full_name.starts_with("@file ") {
                        if self.chunks.contains_key(&full_name) && !is_replace {
                            let location = ChunkLocation {
                                file_idx,
                                line: line_no as usize,
                            };
                            let _err_msg = format!(
                                "Chunk error: {}",
                                AzadiError::FileChunkRedefinition {
                                    file_chunk: full_name.clone(),
                                    file_name: self
                                        .file_names
                                        .get(file_idx)
                                        .cloned()
                                        .unwrap_or_default(),
                                    line: location.line,
                                }
                            );
                            self.chunks.remove(&full_name);
                            continue;
                        }
                        if is_replace {
                            // remove old definition
                            self.chunks.remove(&full_name);
                        }
                    } else if is_replace {
                        // normal chunk with @replace
                        self.chunks.remove(&full_name);
                    }

                    // Now define the chunk
                    let rc = self
                        .chunks
                        .entry(full_name.clone())
                        .or_insert_with(|| Rc::new(RefCell::new(NamedChunk::new())));
                    let mut borrowed = rc.borrow_mut();
                    let def_idx = borrowed.definitions.len();
                    borrowed.definitions.push(ChunkDef::new(
                        indentation.len(),
                        file_idx,
                        line_no as usize,
                    ));
                    drop(borrowed);

                    current_chunk = Some((full_name, def_idx));
                }
                continue;
            }

            // If it's a closing line
            if self.close_re.is_match(line) {
                current_chunk = None;
                continue;
            }

            // If we're in a chunk, add lines to it
            if let Some((ref cname, idx)) = current_chunk {
                if let Some(rc) = self.chunks.get(cname) {
                    let mut borrowed = rc.borrow_mut();
                    let def = borrowed.definitions.get_mut(idx).unwrap();
                    if line.ends_with('\n') {
                        def.content.push(line.to_string());
                    } else {
                        def.content.push(format!("{}\n", line));
                    }
                }
            }
        }

        // Update file_chunks array
        let mut fc = Vec::new();
        for (name, _) in &self.chunks {
            if name.starts_with("@file ") {
                fc.push(name.clone());
            }
        }
        self.file_chunks = fc;
    }

    /// Increments references on a chunk or returns an error if undefined.
    fn inc_references(&self, chunk_name: &str, location: &ChunkLocation) -> Result<(), AzadiError> {
        if let Some(rc) = self.chunks.get(chunk_name) {
            let mut borrowed = rc.borrow_mut();
            borrowed.references += 1;
            Ok(())
        } else {
            let file_name = self
                .file_names
                .get(location.file_idx)
                .cloned()
                .unwrap_or_default();
            Err(AzadiError::UndefinedChunk {
                chunk: chunk_name.to_string(),
                file_name,
                line: location.line,
            })
        }
    }

    /// Expands chunk references, possibly reversing definitions if @reversed is in the line.
    pub fn expand_with_depth(
        &self,
        chunk_name: &str,
        target_indent: &str,
        depth: usize,
        seen: &mut Vec<(String, ChunkLocation)>,
        reference_location: ChunkLocation,
        reversed_mode: bool,
    ) -> Result<Vec<String>, AzadiError> {
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            let file_name = self
                .file_names
                .get(reference_location.file_idx)
                .cloned()
                .unwrap_or_default();
            return Err(AzadiError::RecursionLimit {
                chunk: chunk_name.to_string(),
                file_name,
                line: reference_location.line,
            });
        }

        // Check recursion
        if seen.iter().any(|(nm, _)| nm == chunk_name) {
            let file_name = self
                .file_names
                .get(reference_location.file_idx)
                .cloned()
                .unwrap_or_default();
            return Err(AzadiError::RecursiveReference {
                chunk: chunk_name.to_string(),
                file_name,
                line: reference_location.line,
            });
        }

        // Bump references
        self.inc_references(chunk_name, &reference_location)?;

        let rc = match self.chunks.get(chunk_name) {
            Some(r) => r,
            None => {
                let file_name = self
                    .file_names
                    .get(reference_location.file_idx)
                    .cloned()
                    .unwrap_or_default();
                return Err(AzadiError::UndefinedChunk {
                    chunk: chunk_name.to_string(),
                    file_name,
                    line: reference_location.line,
                });
            }
        };

        let borrowed = rc.borrow();
        let defs = &borrowed.definitions;

        // Reverse definitions if @reversed
        let iter: Box<dyn Iterator<Item = &ChunkDef>> = if reversed_mode {
            Box::new(defs.iter().rev())
        } else {
            Box::new(defs.iter())
        };

        seen.push((chunk_name.to_string(), reference_location));
        let mut result = Vec::new();

        for def in iter {
            let mut def_output = Vec::new();
            let mut line_count = 0;
            for line in &def.content {
                line_count += 1;
                // Check if line references another chunk
                if let Some(caps) = self.slot_re.captures(line) {
                    let add_indent = caps.get(1).map_or("", |m| m.as_str());
                    let referenced_chunk = caps.get(2).map_or("", |m| m.as_str());

                    let line_is_reversed = line.contains("@reversed");
                    let relative_indent = if add_indent.len() > def.base_indent {
                        &add_indent[def.base_indent..]
                    } else {
                        ""
                    };
                    let new_indent = if target_indent.is_empty() {
                        relative_indent.to_owned()
                    } else {
                        format!("{}{}", target_indent, relative_indent)
                    };
                    let new_loc = ChunkLocation {
                        file_idx: def.file_idx,
                        line: def.line + line_count - 1,
                    };

                    let expanded = self.expand_with_depth(
                        referenced_chunk.trim(),
                        &new_indent,
                        depth + 1,
                        seen,
                        new_loc,
                        line_is_reversed,
                    )?;
                    def_output.extend(expanded);
                } else {
                    // Plain line
                    let line_indent = if line.len() > def.base_indent {
                        &line[def.base_indent..]
                    } else {
                        line
                    };
                    if target_indent.is_empty() {
                        def_output.push(line_indent.to_owned());
                    } else {
                        def_output.push(format!("{}{}", target_indent, line_indent));
                    }
                }
            }
            result.extend(def_output);
        }

        seen.pop();
        Ok(result)
    }

    /// Expand from top-level (no reversed).
    pub fn expand(&self, chunk_name: &str, indent: &str) -> Result<Vec<String>, AzadiError> {
        let mut seen = Vec::new();
        let loc = ChunkLocation {
            file_idx: 0,
            line: 0,
        };
        self.expand_with_depth(chunk_name, indent, 0, &mut seen, loc, false)
    }

    /// For tests or direct usage: get chunk content with no indentation.
    pub fn get_chunk_content(&self, chunk_name: &str) -> Result<Vec<String>, AzadiError> {
        self.expand(chunk_name, "")
    }

    /// Return a slice of chunk names that start with "@file ".
    pub fn get_file_chunks(&self) -> &[String] {
        &self.file_chunks
    }

    /// Check if the store has a chunk of the given name.
    pub fn has_chunk(&self, name: &str) -> bool {
        self.chunks.contains_key(name)
    }

    /// Reset everything
    pub fn reset(&mut self) {
        self.chunks.clear();
        self.file_chunks.clear();
        self.file_names.clear();
    }

    /// Warnings for any chunk never referenced.
    pub fn check_unused_chunks(&self) -> Vec<String> {
        let mut warns = Vec::new();
        for (name, rc) in &self.chunks {
            if !name.starts_with("@file ") {
                let borrowed = rc.borrow();
                if borrowed.references == 0 {
                    if let Some(first_def) = borrowed.definitions.first() {
                        let fname = self
                            .file_names
                            .get(first_def.file_idx)
                            .cloned()
                            .unwrap_or_default();
                        let ln = first_def.line + 1;
                        warns.push(format!(
                            "Warning: {} line {}: chunk '{}' is defined but never referenced",
                            fname, ln, name
                        ));
                    }
                }
            }
        }
        warns.sort();
        warns
    }
}

/// Writes @file ... chunks to disk
pub struct ChunkWriter<'a> {
    safe_file_writer: &'a mut SafeFileWriter,
}

impl<'a> ChunkWriter<'a> {
    pub fn new(sw: &'a mut SafeFileWriter) -> Self {
        Self {
            safe_file_writer: sw,
        }
    }

    pub fn write_chunk(&mut self, chunk_name: &str, content: &[String]) -> Result<(), AzadiError> {
        if !chunk_name.starts_with("@file ") {
            return Ok(());
        }
        let path_str = &chunk_name[5..].trim();
        let final_path = self.safe_file_writer.before_write(path_str)?;
        let mut f = fs::File::create(&final_path)?;
        for line in content {
            f.write_all(line.as_bytes())?;
        }
        self.safe_file_writer.after_write(path_str)?;
        Ok(())
    }
}

/// High-level reading, expanding, writing API.
pub struct Clip {
    store: ChunkStore,
    writer: SafeFileWriter,
}

impl Clip {
    pub fn new(
        safe_file_writer: SafeFileWriter,
        open_delim: &str,
        close_delim: &str,
        chunk_end: &str,
        comment_markers: &[String],
    ) -> Self {
        Self {
            store: ChunkStore::new(open_delim, close_delim, chunk_end, comment_markers),
            writer: safe_file_writer,
        }
    }

    pub fn reset(&mut self) {
        self.store.reset();
    }

    pub fn has_chunk(&self, name: &str) -> bool {
        self.store.has_chunk(name)
    }

    pub fn get_file_chunks(&self) -> Vec<String> {
        self.store.get_file_chunks().to_vec()
    }

    pub fn check_unused_chunks(&self) -> Vec<String> {
        self.store.check_unused_chunks()
    }

    /// Read from a file on disk, storing chunk definitions.
    pub fn read_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), AzadiError> {
        let fname = path.as_ref().to_string_lossy().to_string();
        let idx = self.store.add_file_name(&fname);
        let text = fs::read_to_string(&path)?;
        self.store.read(&text, idx);
        Ok(())
    }

    /// Read from an in-memory string, specifying a "filename" for error messages.
    pub fn read(&mut self, text: &str, file_name: &str) {
        let idx = self.store.add_file_name(file_name);
        self.store.read(text, idx);
    }

    /// Write all file chunks to disk.
    pub fn write_files(&mut self) -> Result<(), AzadiError> {
        let fc = self.store.get_file_chunks().to_vec();
        for name in &fc {
            let expanded = self.store.expand(name, "")?;
            let mut cw = ChunkWriter::new(&mut self.writer);
            cw.write_chunk(name, &expanded)?;
        }
        let warns = self.store.check_unused_chunks();
        for w in warns {
            eprintln!("{}", w);
        }
        Ok(())
    }

    /// Expand a chunk and write to an arbitrary writer.
    pub fn get_chunk<W: io::Write>(
        &self,
        chunk_name: &str,
        out_stream: &mut W,
    ) -> Result<(), AzadiError> {
        let lines = self.store.expand(chunk_name, "")?;
        for line in lines {
            out_stream.write_all(line.as_bytes())?;
        }
        out_stream.write_all(b"\n")?;
        Ok(())
    }

    /// Expand a chunk into a vector of lines.
    pub fn expand(&self, chunk_name: &str, indent: &str) -> Result<Vec<String>, AzadiError> {
        Ok(self.store.expand(chunk_name, indent)?)
    }

    /// Retrieve the chunk content directly (commonly used in tests).
    pub fn get_chunk_content(&self, name: &str) -> Result<Vec<String>, AzadiError> {
        self.store.get_chunk_content(name)
    }

    pub fn read_files<P: AsRef<Path>>(&mut self, input_paths: &[P]) -> Result<(), AzadiError> {
        for path in input_paths {
            self.read_file(path)?;
        }
        Ok(())
    }
}
// $$
