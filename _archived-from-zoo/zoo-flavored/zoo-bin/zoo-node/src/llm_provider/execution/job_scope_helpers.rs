use crate::llm_provider::job_manager::JobManager;
use zoo_fs::zoo_file_manager::ZooFileManager;
use zoo_message_primitives::schemas::zoo_fs::ZooFileChunkCollection;
use zoo_message_primitives::zoo_utils::job_scope::MinimalJobScope;
use zoo_message_primitives::zoo_utils::zoo_logging::{zoo_log, ZooLogLevel, ZooLogOption};
use zoo_message_primitives::zoo_utils::zoo_path::ZooPath;
use zoo_sqlite::errors::SqliteManagerError;
use zoo_sqlite::SqliteManager;
use std::result::Result::Ok;
use std::collections::HashMap;

impl JobManager {
    /// Retrieves all resources in the given job scope and returns them as a vector of ZooFileChunkCollection.
    pub async fn retrieve_all_resources_in_job_scope(
        scope: &MinimalJobScope,
        sqlite_manager: &SqliteManager,
    ) -> Result<Vec<ZooFileChunkCollection>, SqliteManagerError> {
        let mut collections = Vec::new();

        // Retrieve each file in the job scope
        for path in &scope.vector_fs_items {
            if let Some(collection) = JobManager::retrieve_file_chunks(path, sqlite_manager).await? {
                collections.push(collection);
            }
        }

        // Retrieve files inside vector_fs_folders
        for folder in &scope.vector_fs_folders {
            let files = match ZooFileManager::list_directory_contents(folder.clone(), sqlite_manager) {
                Ok(files) => files,
                Err(e) => {
                    zoo_log(
                        ZooLogOption::JobExecution,
                        ZooLogLevel::Error,
                        &format!("Error listing directory contents: {:?}", e),
                    );
                    return Err(SqliteManagerError::SomeError(format!("ZooFsError: {:?}", e)));
                }
            };

            for file_info in files {
                if !file_info.is_directory && file_info.has_embeddings {
                    let file_path = ZooPath::from_string(file_info.path);
                    if let Some(collection) = JobManager::retrieve_file_chunks(&file_path, sqlite_manager).await? {
                        collections.push(collection);
                    }
                }
            }
        }

        Ok(collections)
    }

    /// Static function to retrieve file chunks for a given path.
    pub async fn retrieve_file_chunks(
        path: &ZooPath,
        sqlite_manager: &SqliteManager,
    ) -> Result<Option<ZooFileChunkCollection>, SqliteManagerError> {
        match sqlite_manager.get_parsed_file_by_zoo_path(path) {
            Ok(Some(parsed_file)) if parsed_file.embedding_model_used.is_some() => {
                let chunks = sqlite_manager.get_chunks_for_parsed_file(parsed_file.id.unwrap())?;
                let mut paths_map = HashMap::new();
                paths_map.insert(parsed_file.id.unwrap(), path.clone());
                Ok(Some(ZooFileChunkCollection { chunks, paths: Some(paths_map) }))
            }
            Ok(Some(_)) => {
                zoo_log(
                    ZooLogOption::JobExecution,
                    ZooLogLevel::Info,
                    &format!("File has no embeddings: {}", path),
                );
                Ok(None)
            }
            Ok(None) => {
                zoo_log(
                    ZooLogOption::JobExecution,
                    ZooLogLevel::Error,
                    &format!("File not found in database: {}", path),
                );
                Ok(None)
            }
            Err(e) => {
                zoo_log(
                    ZooLogOption::JobExecution,
                    ZooLogLevel::Error,
                    &format!("Error retrieving file from database: {} with error: {:?}", path, e),
                );
                Err(e)
            }
        }
    }
}

// TODO: implement tests under a cfg. 