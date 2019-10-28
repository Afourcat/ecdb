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
use super::AttributeType;
use super::Schema;
use byteorder::{BigEndian, ByteOrder};
use std::collections::HashMap;

#[derive(Debug)]
enum ComponentAttributeValue {
    String(String),
    Integer(i128),
    Float(f64),
}

impl ComponentAttributeValue {
    fn parse_attribute_type(
        attribute_type: &AttributeType,
        data: Vec<u8>,
    ) -> Result<Self, &'static str> {
        Ok(match attribute_type {
            AttributeType::String => {
                Self::String(String::from_utf8(data).map_err(|_| "Cannot convert to UTF8")?)
            }
            AttributeType::Integer => Self::Integer(BigEndian::read_i128(&data)),
            AttributeType::Float => Self::Float(BigEndian::read_f64(&data)),
        })
    }
}

impl Default for ComponentAttributeValue {
    fn default() -> Self {
        Self::String(String::from("null"))
    }
}

#[derive(Debug, Default)]
struct ComponentAttribute {
    pub name: String,
    pub value: ComponentAttributeValue,
}

impl ComponentAttribute {
    fn parse_component_attribute(
        component_schema: &HashMap<String, AttributeType>,
        component_value: protos::entity::ComponentValue,
    ) -> Result<Self, &'static str> {
        match component_schema.get(&component_value.name) {
            Some(attribute_type) => Ok(Self {
                name: component_value.name,
                value: ComponentAttributeValue::parse_attribute_type(
                    attribute_type,
                    component_value.value,
                )?,
            }),
            None => Err("Cannot get the attribute type for this component attribute"),
        }
    }
}

#[derive(Debug, Default)]
struct Component {
    pub name: String,
    attributes: Vec<ComponentAttribute>,
}

impl Component {
    fn parse_component(
        schema: &Schema,
        component: protos::entity::Component,
    ) -> Result<Self, &'static str> {
        let component_schema = schema
            .components
            .get(&component.name)
            .ok_or("Cannot get the schema for this component.")?;

        Ok(Self {
            name: component.name,
            attributes: component
                .values
                .into_iter()
                .map(|value| ComponentAttribute::parse_component_attribute(component_schema, value))
                .collect::<Result<Vec<ComponentAttribute>, &'static str>>()?,
        })
    }
}

#[derive(Debug, Default)]
struct Entity {
    components: Vec<Component>,
}
