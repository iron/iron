use std::io::IoResult;

use super::{get_reader, get_writer};

pub fn generate(list: Path, module: Path) -> IoResult<()> {
    let mut reader = get_reader(list);
    let mut writer = get_writer(module);

    try!(writer.write(b"\
// This automatically generated file is included in response.rs.
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
"));

    /* Generated snippets will look like:
    "json" => Some(MediaType {
        type_: "application".to_str(),
        subtype: "json".to_str(),
        parameters: vec![]
    }),
    */

    // loop over lines
        // loop over fields
            // populate the enum


writer.write(b"        _ => None
    }
}
")
}
