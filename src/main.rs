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

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

#[macro_use]
extern crate serde;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::fs;
use std::path::PathBuf;
mod protos;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::{Read};
use protobuf::{parse_from_bytes};
use protobuf::Message;

const FILENAME: &'static str = "schema.json";

mod entity;
mod schema;

use schema::{Schema, AttributeType};

fn handle_client(mut stream: TcpStream, schema: Arc<Mutex<Schema>>) {
    let mut data = [0 as u8; 1024]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            match size {
                0 => false,
                _ => {
                    match parse_from_bytes::<protos::component_schema::AddComponentSchema>(&data[0..size]) {
                        Ok(parsed) => {
                            let proto_schema = parsed.schema.unwrap();
                            let name = String::from(&proto_schema.name);
                            schema.lock().unwrap().add_component(&name, HashMap::from(proto_schema));
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
    let mut ent = protos::entity::CreateEntity::default();
    let mut comp = protos::entity::Component::default();
    let mut x = protos::entity::ComponentValue::default();
    x.name = String::from("x");
    x.value.write_u64::<BigEndian>(517).unwrap();
    let mut y = protos::entity::ComponentValue::default();
    y.name = String::from("y");
    y.value.write_u64::<BigEndian>(12).unwrap();
    let mut z = protos::entity::ComponentValue::default();
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
    match schema.lock().unwrap().write_schema(PathBuf::from(FILENAME)) {
        Ok(_) => (),
        Err(err) => {
            println!("Error: {:?}", err);
        }
    };
}
