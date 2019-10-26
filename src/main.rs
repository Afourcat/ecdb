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

#[macro_use]
extern crate serde;

use std::sync::{Mutex, Arc};
use std::fs;
use std::path::PathBuf;
mod protos;
use std::net::{TcpListener, TcpStream, Shutdown};
use serde::{Deserialize, Serialize};
use std::thread;
use std::io::{Read};
use protobuf::parse_from_bytes;

const FILENAME: &'static str = "schema.json";

#[derive(Serialize, Deserialize, Debug)]
enum AttributeType {
    String,
    Integer,
    Float
}


#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    pub name: String,
    pub field_type: AttributeType
}


#[derive(Serialize, Deserialize, Debug)]
struct ComponentSchema {
    pub name: String,
    pub attributes: Vec<Attribute>
}

impl From<protos::component_schema::ComponentSchema> for ComponentSchema
{

    fn from(schema: protos::component_schema::ComponentSchema) -> Self {
        ComponentSchema {
            name: schema.name,
            attributes: schema.attributes.into_iter().map(|attr| {
                Attribute {
                    name: attr.name,
                    field_type: match attr.field_type {
                        protos::component_schema::AttributeType::Integer => AttributeType::Integer,
                        protos::component_schema::AttributeType::String => AttributeType::String,
                        protos::component_schema::AttributeType::Float => AttributeType::Float,
                    }
                }
            }).collect()
        }
    }

}


#[derive(Serialize, Deserialize, Debug, Default)]
struct Schema {
    pub schema: Vec<ComponentSchema>
}

impl Schema {

    pub fn add_component(&mut self, component: ComponentSchema) {
        self.schema.push(component);
    }

    fn write_schema(&self, filename: PathBuf) -> Result<(), &'static str> {
        match serde_json::to_string(self) {
            Ok(content) => fs::write(filename, content).map_err(|_| "Canno write to file"),
            Err(_err) => Err("Cannot convert to JSON.")
        }
    }

}

impl From<PathBuf> for Schema {

    fn from(filename: PathBuf) -> Self {
        match fs::read_to_string(filename) {
            Ok(contents) => {
                match serde_json::from_str::<Schema>(&contents) {
                    Ok(schema) => schema,
                    Err(_err) => Schema::default()
                }
            },
            Err(_err) => Schema::default()
        }
    }

}

fn handle_client(mut stream: TcpStream, schema: Arc<Mutex<Schema>>) {
    let mut data = [0 as u8; 1024]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            match size {
                0 => false,
                _ => {
                    match parse_from_bytes::<protos::component_schema::AddComponentSchema>(&data[0..size]) {
                        Ok(parsed) => {
                            schema.lock().unwrap().add_component(ComponentSchema::from(parsed.schema.unwrap()));
                            schema.lock().unwrap().write_schema(PathBuf::from(FILENAME));
                        },
                        Err(_) => ()
                    }
                    true
                }
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    let schema = Arc::new(Mutex::new(Schema::from(PathBuf::from(FILENAME))));
    let listener = TcpListener::bind("0.0.0.0:8687").unwrap();
    println!("Listening on port 8687");
    for stream in listener.incoming() {
        let schema = schema.clone();
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    handle_client(stream, schema)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);

    match schema.lock().unwrap().write_schema(PathBuf::from(FILENAME)) {
        Ok(_) => (),
        Err(err) => {
            println!("Error: {:?}", err);
        }
    };
}
