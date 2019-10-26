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

extern crate protobuf_codegen_pure;

fn main() {

    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/protos",
        input: &["protos/component_schema.proto", "protos/entity.proto"],
        includes: &["protos"],
        customize: protobuf_codegen_pure::Customize {
            serde_derive: Some(true),
            ..Default::default()
        },
    }).expect("protoc");
}