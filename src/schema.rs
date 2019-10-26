// Copyright 2019 Thomas Nicollet
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::protos;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum AttributeType {
    Integer,
    String,
    Float,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Schema {
    pub components: HashMap<String, HashMap<String, AttributeType>>,
}

impl From<protos::component_schema::AttributeType> for AttributeType {

    fn from(attribute_type: protos::component_schema::AttributeType) -> Self {
        match attribute_type {
            protos::component_schema::AttributeType::Integer => Self::Integer,
            protos::component_schema::AttributeType::String => Self::String,
            protos::component_schema::AttributeType::Float => Self::Float,
        }
    }

}

impl From<protos::component_schema::ComponentSchema> for HashMap<String, AttributeType> {
    fn from(schema: protos::component_schema::ComponentSchema) -> Self {
        schema
            .attributes
            .into_iter()
            .map(|attr| (attr.name, AttributeType::from(attr.field_type)))
            .collect()
    }
}

impl Schema {
    pub fn add_component(
        &mut self,
        name: &str,
        component_attributes: HashMap<String, AttributeType>,
    ) {
        self.components
            .insert(String::from(name), component_attributes);
    }

    pub fn write_schema(&self, filename: PathBuf) -> Result<(), &'static str> {
        match serde_json::to_string(self) {
            Ok(content) => fs::write(filename, content).map_err(|_| "Cannot write to file"),
            Err(_err) => Err("Cannot convert to JSON."),
        }
    }
}

impl From<PathBuf> for Schema {
    fn from(filename: PathBuf) -> Self {
        match fs::read_to_string(filename) {
            Ok(contents) => match serde_json::from_str::<Schema>(&contents) {
                Ok(schema) => schema,
                Err(_err) => Schema::default(),
            },
            Err(_err) => Schema::default(),
        }
    }
}
