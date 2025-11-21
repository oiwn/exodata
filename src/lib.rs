pub mod app;
#[cfg(feature = "ssr")]
pub mod common;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;
#[cfg(feature = "ssr")]
pub mod stellarhosts;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    use leptos::mount::mount_to_body;
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[cfg(test)]
mod tests {
    use struct_macro::ImplNew;
    use votable::impls::VOTableValue;

    #[derive(Debug, ImplNew)]
    struct TestStruct {
        field1: Option<String>,
        field2: Option<f64>,
        field3: Option<i32>,
        field4: Option<String>, // For CharASCII and CharUnicode
    }
    #[test]
    fn test_new_function() {
        let row = vec![
            VOTableValue::String("TestString".to_string()),
            VOTableValue::Double(42.0),
            VOTableValue::Int(7),
            VOTableValue::CharASCII('a'),
        ];

        let test_struct = TestStruct::new(row);

        assert_eq!(test_struct.field1, Some("TestString".to_string()));
        assert_eq!(test_struct.field2, Some(42.0));
        assert_eq!(test_struct.field3, Some(7));
        assert_eq!(test_struct.field4, Some("a".to_string()));
    }

    #[test]
    fn test_null_values() {
        let row = vec![
            VOTableValue::Null,
            VOTableValue::Double(42.0),
            VOTableValue::Null,
            VOTableValue::CharUnicode('b'),
        ];

        let test_struct = TestStruct::new(row);

        assert_eq!(test_struct.field1, None);
        assert_eq!(test_struct.field2, Some(42.0));
        assert_eq!(test_struct.field3, None);
        assert_eq!(test_struct.field4, Some("b".to_string()));
    }
}
