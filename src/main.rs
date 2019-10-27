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

mod protos;

use byteorder::{BigEndian, WriteBytesExt};

extern crate serde;
use protobuf::parse_from_bytes;
use protobuf::Message;
use protos::{
    component_schema::AddComponentSchema,
    entity::{Component, ComponentValue, CreateEntity},
};
use std::collections::HashMap;
use std::io::Read;
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const FILENAME: &str = "schema.json";

mod entity;
mod schema;

use schema::{AttributeType, Schema};

fn handle_client(mut stream: TcpStream, schema: Arc<Mutex<Schema>>) {
    let mut data = [0 as u8; 1024]; // using 50 byte buffer

    while match stream.read(&mut data) {
        Ok(0) => false,
        Ok(size) => {
            if let Ok(parsed) = parse_from_bytes::<AddComponentSchema>(&data[0..size]) {
                let proto_schema = parsed.schema.unwrap();
                let name = String::from(&proto_schema.name);
                schema
                    .lock()
                    .unwrap()
                    .add_component(&name, HashMap::from(proto_schema));
                if let Err(err) = schema.lock().unwrap().write_schema(PathBuf::from(FILENAME)) {
                    eprintln!("An error occured {}", err);
                }
            }
            true
        }
        Err(err) => {
            eprintln!(
                "An error occurred, terminating connection with {}, {}",
                stream.peer_addr().unwrap(),
                err
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    let schema = Arc::new(Mutex::new(Schema::from(PathBuf::from(FILENAME))));
    let mut ent = CreateEntity::default();
    let mut comp = Component::default();
    let mut x = ComponentValue::default();
    x.name = String::from("x");
    x.value.write_u64::<BigEndian>(517).unwrap();
    let mut y = ComponentValue::default();
    y.name = String::from("y");
    y.value.write_u64::<BigEndian>(12).unwrap();
    let mut z = ComponentValue::default();
    z.name = String::from("z");
    z.value.write_u64::<BigEndian>(25).unwrap();
    comp.name = String::from("Velocity");
    comp.values.push(x);
    comp.values.push(y);
    comp.values.push(z);
    ent.components.push(comp);

    println!("{:?}", ent.compute_size());
    println!("{:?}", ent);

    /*let listener = TcpListener::bind("0.0.0.0:8687").unwrap();
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
    */
    if let Err(err) = schema.lock().unwrap().write_schema(PathBuf::from(FILENAME)) {
        eprintln!("Error: {:?}", err);
    };
}
