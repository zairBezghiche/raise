use crate::arcadia_element;
use crate::model_engine::common::ElementRef;

// --- Configuration Item ---
arcadia_element!(ConfigurationItem {
    kind: String, // "Hardware", "Software", "SystemPart", ...

    #[serde(rename = "partNumber", skip_serializing_if = "Option::is_none")]
    part_number: Option<String>,

    #[serde(rename = "versionId", skip_serializing_if = "Option::is_none")]
    version_id: Option<String>,

    #[serde(default)]
    composition: Vec<ElementRef>,

    #[serde(rename = "allocatedPhysicalArtifacts", default)]
    allocated_physical_artifacts: Vec<ElementRef>
});
