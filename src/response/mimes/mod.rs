use http::headers::content_type::MediaType;

use self::mimegen::get_generated_content_type;

mod mimegen;

pub fn get_content_type(path: &Path) -> Option<MediaType> {
    path.extension_str().and_then(get_generated_content_type)
}

#[test]
fn matches_content_type () {
    let path = &Path::new("test.txt");
    let content_type = get_content_type(path).unwrap();

    assert_eq!(content_type.type_.as_slice(), "text");
    assert_eq!(content_type.subtype.as_slice(), "plain");
}