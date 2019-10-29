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
#[cfg_attr(feature = "with-serde", macro_use)]
#[cfg(feature = "with-serde")]
extern crate serde;

mod protos;

use byteorder::{BigEndian, WriteBytesExt};

use protobuf::parse_from_bytes;
use protobuf::Message;
use protos::{
    component_schema::AddComponentSchema,
    entity::{Component, ComponentValue, CreateEntity},
};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::Read;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bincode;
use std::fmt::Debug;
use std::iter::FromIterator;

const FILENAME: &str = "schema.json";

mod entity;
mod schema;

use schema::{AttributeType, Schema};
use std::thread;

pub fn get_entities_in_all_map<T: Clone + Hash + Eq + Debug, V>(maps: Vec<HashMap<T, V>>) {
    let mut keys = maps
        .iter()
        .map(|r| HashSet::from_iter(r.keys()))
        .collect::<Vec<HashSet<&T>>>();

    keys.sort_by(|key1, key2| key1.len().cmp(&key2.len()));

    let mut iter = keys.into_iter();
    let intersection = iter
        .next()
        .map(|set| iter.fold(set, |set1 , set2| &set1 & &set2));
    println!("{:?}", intersection);
}

#[allow(dead_code)]
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
    x.value.write_i128::<BigEndian>(517).unwrap();
    let mut y = ComponentValue::default();
    y.name = String::from("y");
    y.value.write_i128::<BigEndian>(12).unwrap();
    let mut z = ComponentValue::default();
    z.name = String::from("z");
    z.value.write_i128::<BigEndian>(25).unwrap();
    comp.name = String::from("Velocity");
    comp.values.push(x);
    comp.values.push(y);
    comp.values.push(z);
    ent.components.push(comp);

    let position = HashMap::from_iter(vec![
        (String::from("A"), ()),
        (String::from("C"), ()),
        (String::from("E"), ()),
        (String::from("F"), ()),
        (String::from("H"), ()),
    ]);
    let velocity = HashMap::from_iter(vec![
        (String::from("C"), ()),
        (String::from("D"), ()),
        (String::from("I"), ()),
        (String::from("E"), ()),
        (String::from("H"), ()),
    ]);
    let explosive = HashMap::from_iter(vec![
        (String::from("A"), ()),
        (String::from("E"), ()),
        (String::from("I"), ()),
        (String::from("Z"), ()),
        (String::from("H"), ()),
    ]);

    get_entities_in_all_map(vec![position, velocity, explosive]);

    for component in ent.components.into_iter() {
        println!("{:?}", component.compute_size());
        let comp = entity::Component::parse_component(&schema.lock().unwrap(), component).unwrap();
        let encoded = bincode::serialize(&comp).unwrap();
        println!("{:?}", encoded.len());
        println!("{:?}", comp);
        println!("{:?}", encoded);
    }

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

    drop(listener);*/
    if let Err(err) = schema.lock().unwrap().write_schema(PathBuf::from(FILENAME)) {
        eprintln!("Error: {:?}", err);
    };
}
