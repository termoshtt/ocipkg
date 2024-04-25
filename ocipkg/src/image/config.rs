use crate::{error::*, Digest};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// The contents of `application/vnd.ocipkg.v1.config+json` media type.
///
/// This is a map from the layer digest to the list of relative paths of the files in the layer.
///
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    layers: HashMap<Digest, Vec<PathBuf>>,
}

impl Config {
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn add_layer(&mut self, digest: Digest, paths: Vec<PathBuf>) {
        self.layers.insert(digest, paths);
    }
}
