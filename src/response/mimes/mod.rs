use std::path::BytesContainer;

use http::headers::content_type::MediaType;

use self::mimegen::get_generated_content_type;

mod mimegen;

pub fn get_content_type(path: &Path) -> Option<MediaType> {
    let path_str = path.container_as_str().unwrap();
    let ext_pos = regex!(".[a-z0-9]+$").find(path_str);
    let mut ext;
    match ext_pos {
        Some((start, _)) => {
            ext = path_str.as_slice().slice_from(start);
        },
        None => return None
    }

    get_generated_content_type(ext)
}
