use std::ops::Deref;

use flexbuffers;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tracing::warn;

use crate::database::Database;
use crate::utils::error::Error;
use crate::world::chunkformat::Chunk;

impl Database {
    /// Inserts a chunk into the database for a given dimension.
    ///
    /// # Arguments
    ///
    /// * `value` - The chunk to be inserted.
    /// * `dimension` - The dimension in which the chunk is located.
    ///
    /// # Returns
    ///
    /// * `Result<bool, Error>` - Returns `Ok(true)` if the chunk already exists, `Ok(false)` otherwise.
    ///
    /// # Errors
    ///
    /// * Returns an `Error` if the insertion fails.
    pub async fn insert_chunk(&self, value: Chunk, dimension: &str) -> Result<bool, Error> {
        let db = self.db.clone();
        let record_name = format!("{},{}", value.x_pos, value.z_pos);
        let tree_name = format!("chunks/{}", dimension);
        let result = tokio::task::spawn_blocking(move || {
            let mut ser = flexbuffers::FlexbufferSerializer::new();
            value.serialize(&mut ser).unwrap();
            let encoded = ser.take_buffer();
            db.open_tree(tree_name)
                .unwrap()
                .insert(record_name, encoded)
        })
        .await
        .expect("Failed to join tasks")
        .expect("Failed to insert chunk");
        match result {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Retrieves a chunk from the database for a given dimension and coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the chunk.
    /// * `z` - The z-coordinate of the chunk.
    /// * `dimension` - The dimension in which the chunk is located.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Chunk>, Error>` - Returns `Ok(Some(chunk))` if the chunk was found, `Ok(None)` otherwise.
    ///
    /// # Errors
    ///
    /// * Returns an `Error` if the retrieval fails.
    pub async fn get_chunk(&self, x: i32, z: i32, dimension: &str) -> Result<Option<Chunk>, Error> {
        let db = self.db.clone();
        let tree_name = format!("chunks/{}", dimension);
        let result = tokio::task::spawn_blocking(move || {
            let record_name = format!("{},{}", x, z);
            let chunk = db.open_tree(tree_name).unwrap().get(record_name).unwrap();
            match chunk {
                Some(chunk) => {
                    let chunk = chunk.as_ref();
                    let deserializer = flexbuffers::Reader::get_root(chunk).unwrap();
                    let chunk: Chunk = Chunk::deserialize(deserializer).unwrap();
                    Some(chunk)
                }
                None => None,
            }
        })
        .await
        .expect("Failed to join tasks");
        Ok(result)
    }

    /// Checks if a chunk exists in the database for a given dimension and coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the chunk.
    /// * `z` - The z-coordinate of the chunk.
    /// * `dimension` - The dimension in which the chunk is located.
    ///
    /// # Returns
    ///
    /// * `Result<bool, Error>` - Returns `Ok(true)` if the chunk exists, `Ok(false)` otherwise.
    ///
    /// # Errors
    ///
    /// * Returns an `Error` if the check fails.
    pub async fn chunk_exists(&self, x: i32, z: i32, dimension: String) -> Result<bool, Error> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let record_name = format!("{},{}", x, z);
            db.open_tree(format!("chunks/{}", dimension))
                .unwrap()
                .contains_key(record_name)
        })
        .await
        .expect("Failed to join tasks")
        .expect("Failed to check if chunk exists");
        Ok(result)
    }

    /// Updates a chunk in the database for a given dimension.
    ///
    /// # Arguments
    ///
    /// * `value` - The chunk to be updated.
    /// * `dimension` - The dimension in which the chunk is located.
    ///
    /// # Returns
    ///
    /// * `Result<bool, Error>` - Returns `Ok(true)` if the chunk was updated, `Ok(false)` otherwise.
    ///
    /// # Errors
    ///
    /// * Returns an `Error` if the update fails.
    pub async fn update_chunk(&self, value: Chunk, dimension: String) -> Result<bool, Error> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let record_name = format!("{},{}", value.x_pos, value.z_pos);
            let mut ser = flexbuffers::FlexbufferSerializer::new();
            value.serialize(&mut ser).unwrap();
            let encoded = ser.take_buffer();
            if db
                .open_tree("chunks")
                .unwrap()
                .remove(&record_name)
                .unwrap()
                .is_none()
            {
                warn!("Attempted to update non-existent chunk: {}", record_name);
            }
            db.open_tree(format!("chunks/{}", dimension))
                .unwrap()
                .insert(record_name, encoded)
        })
        .await
        .expect("Failed to join tasks")
        .expect("Failed to update chunk");
        match result {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Retrieves a range of chunks from the database for a given dimension and coordinates.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting coordinates (x, z) of the range.
    /// * `end` - The ending coordinates (x, z) of the range.
    /// * `dimension` - The dimension in which the chunks are located.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Option<Chunk>>, Error>` - Returns a vector of chunks within the specified range.
    ///
    /// # Errors
    ///
    /// * Returns an `Error` if the retrieval fails.
    pub async fn get_chunk_range(
        &self,
        start: (i32, i32),
        end: (i32, i32),
        dimension: &str,
    ) -> Result<Vec<Option<Chunk>>, Error> {
        // TODO: Make this work
        // let mut set = JoinSet::new();
        //
        // for x in start.0..end.0 {
        //     for z in start.1..end.1 {
        //         set.spawn(async move { self.get_chunk(x, z, dimension).await.unwrap() });
        //     }
        // }
        //
        // let mut results = Vec::new();
        //
        // while let Some(result) = set.join_next().await {
        //     results.push(result?);
        // }

        let mut results = Vec::new();

        for x in start.0..end.0 {
            for z in start.1..end.1 {
                let result = self.get_chunk(x, z, dimension).await?;
                results.push(result);
            }
        }

        Ok(results)
    }
}