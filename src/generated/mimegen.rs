use std::io::IoResult;
use std::collections::hashmap::HashMap;
use std::str::from_utf8;

use super::{get_file_reader, get_file_writer};

pub fn generate(list: Path, module: Path) -> IoResult<()> {
    let mut reader = get_file_reader(list);
    let mut writer = get_file_writer(module);

    try!(writer.write(
b"// This automatically generated file is included in response.rs.
use std::path::BytesContainer;

use http::headers::content_type::MediaType;

pub fn get_content_type(path: &Path) -> Option<MediaType> {
    let path_str = path.container_as_str().unwrap();
    let ext_pos = regex!(\".[a-z0-9]+$\").find(path_str);
    let mut ext;
    match ext_pos {
        Some((start, _)) => {
            ext = path_str.as_slice().slice_from(start);
        },
        None => return None
    }

    match ext {
"   ));

    /* Generated snippets will look like:
    "json" => Some(MediaType {
        type_: "application".to_str(),
        subtype: "json".to_str(),
        parameters: vec![]
    }),
    */

    let mut byter = reader.bytes();
    // avoid duplicates
    let mut seen = HashMap::new();
    'read: loop {
        let mut ext = vec![];
        let mut type_ = vec![];
        let mut subtype = vec![];
        loop {
            match byter.next() {
                Some(Ok(b' ')) => break,
                Some(Ok(c)) => ext.push(c),
                Some(Err(e)) => return Err(e),
                None => break 'read
            }
        }
        loop {
            match byter.next() {
                Some(Ok(b' ')) => break,
                Some(Ok(c)) => type_.push(c),
                Some(Err(e)) => return Err(e),
                None => break 'read
            }
        }
        loop {
            match byter.next() {
                Some(Ok(b'\n')) => break,
                Some(Ok(c)) => subtype.push(c),
                Some(Err(e)) => return Err(e),
                None => break 'read
            }
        }

        if !seen.contains_key(&ext) {

            try!(write!(writer,
"    \"{}\" => Some(MediaType {{
        type_: \"{}\".to_str(),
        subtype: \"{}\".to_str(),
        parameters: vec![]
    }}),\n", from_utf8(ext.as_slice()).unwrap(),
             from_utf8(type_.as_slice()).unwrap(),
             from_utf8(subtype.as_slice()).unwrap()));

            seen.insert(ext, true);
        }
    }

    writer.write(b"        _ => None\n    }\n}\n")
}
