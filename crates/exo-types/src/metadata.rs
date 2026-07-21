use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub datatype: String,
}

#[cfg(test)]
mod tests {
    use super::ColumnMetadata;
    use serde_json::json;

    #[test]
    fn serializes_the_public_metadata_shape() {
        let metadata = ColumnMetadata {
            name: "pl_rade".to_string(),
            description: Some("Planet Radius".to_string()),
            unit: None,
            datatype: "double".to_string(),
        };

        assert_eq!(
            serde_json::to_value(metadata).unwrap(),
            json!({
                "name": "pl_rade",
                "description": "Planet Radius",
                "unit": null,
                "datatype": "double",
            })
        );
    }
}
