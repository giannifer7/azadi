// weaveback-lsp/src/nav.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::client::{LspClient, LspError};
use lsp_types::*;
use std::path::Path;
use url::Url;

impl LspClient {
    pub fn goto_definition(
        &mut self,
        path: &Path,
        line: u32,
        col: u32,
    ) -> Result<Option<Location>, LspError> {
        let uri = Url::from_file_path(path)
            .map_err(|_| LspError::Protocol("invalid file path".into()))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier::new(uri),
            position: Position::new(line, col),
        };

        let res = self.call("textDocument/definition", params)?;
        if res.is_null() { return Ok(None); }

        // Definition can return Location, Vec<Location>, or Vec<LocationLink>
        if let Ok(loc) = serde_json::from_value::<Location>(res.clone()) {
            Ok(Some(loc))
        } else if let Ok(locs) = serde_json::from_value::<Vec<Location>>(res.clone()) {
            Ok(locs.into_iter().next())
        } else {
            // For now, ignore LocationLink and other complex types
            Ok(None)
        }
    }

    pub fn find_references(
        &mut self,
        path: &Path,
        line: u32,
        col: u32,
    ) -> Result<Vec<Location>, LspError> {
        let uri = Url::from_file_path(path)
            .map_err(|_| LspError::Protocol("invalid file path".into()))?;

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier::new(uri),
                position: Position::new(line, col),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        };

        let res = self.call("textDocument/references", params)?;
        if res.is_null() { return Ok(vec![]); }

        let locs: Vec<Location> = serde_json::from_value(res)?;
        Ok(locs)
    }

    pub fn hover(
        &mut self,
        path: &Path,
        line: u32,
        col: u32,
    ) -> Result<Option<Hover>, LspError> {
        let uri = Url::from_file_path(path)
            .map_err(|_| LspError::Protocol("invalid file path".into()))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier::new(uri),
            position: Position::new(line, col),
        };

        let res = self.call("textDocument/hover", params)?;
        if res.is_null() { return Ok(None); }

        let hover: Hover = serde_json::from_value(res)?;
        Ok(Some(hover))
    }

    pub fn document_symbols(
        &mut self,
        path: &Path,
    ) -> Result<Vec<DocumentSymbolResponse>, LspError> {
        let uri = Url::from_file_path(path)
            .map_err(|_| LspError::Protocol("invalid file path".into()))?;

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier::new(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let res = self.call("textDocument/documentSymbol", params)?;
        if res.is_null() { return Ok(vec![]); }

        let symbols: Vec<DocumentSymbolResponse> = serde_json::from_value(res)?;
        Ok(symbols)
    }
}
