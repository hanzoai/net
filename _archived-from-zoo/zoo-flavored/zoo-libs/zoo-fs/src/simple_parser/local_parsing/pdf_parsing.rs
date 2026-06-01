use std::path::PathBuf;

use zoo_non_rust_code::functions::parse_pdf::parse_pdf;

use crate::{
    zoo_fs_error::ZooFsError, simple_parser::{file_parser_helper::ZooFileParser, text_group::TextGroup}
};

use super::LocalFileParser;

impl LocalFileParser {
    pub async fn process_pdf_file(
        file_path: PathBuf,
        max_node_text_size: u64,
    ) -> Result<Vec<TextGroup>, ZooFsError> {
        let parsed_pages = parse_pdf(file_path)
            .await
            .map_err(|_| ZooFsError::FailedPDFParsing)?;

        let mut text_groups = Vec::new();

        for page in parsed_pages.pages {
            ZooFileParser::push_text_group_by_depth(
                &mut text_groups,
                0,
                page.text,
                max_node_text_size,
                Some(page.metadata.page.try_into().unwrap_or_default()),
            );
        }

        Ok(text_groups)
    }
}
