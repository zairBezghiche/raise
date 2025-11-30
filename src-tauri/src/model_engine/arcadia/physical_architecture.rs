use crate::arcadia_element;
use crate::model_engine::common::ElementRef;

// --- Physical Component (Node / Behavior) ---
arcadia_element!(PhysicalComponent {
    nature: String, // "Node" | "Behavior"

    #[serde(rename = "subComponents", default)]
    sub_components: Vec<ElementRef>,

    #[serde(rename = "allocatedFunctions", default)]
    allocated_functions: Vec<ElementRef>,

    #[serde(rename = "realizedLogicalComponents", default)]
    realized_logical_components: Vec<ElementRef>,

    // Pour Behavior
    #[serde(rename = "deployedOn", default)]
    deployed_on: Vec<ElementRef>,

    // Pour Node
    #[serde(rename = "deployedComponents", default)]
    deployed_components: Vec<ElementRef>
});

// --- Physical Function ---
arcadia_element!(PhysicalFunction {
    #[serde(rename = "realizedLogicalFunctions", default)]
    realized_logical_functions: Vec<ElementRef>,

    #[serde(rename = "allocatedTo", default)]
    allocated_to: Vec<ElementRef>,

    #[serde(default)]
    inputs: Vec<ElementRef>,

    #[serde(default)]
    outputs: Vec<ElementRef>
});

// --- Physical Actor ---
arcadia_element!(PhysicalActor {
    #[serde(rename = "realizedLogicalActors", default)]
    realized_logical_actors: Vec<ElementRef>,

    #[serde(rename = "allocatedFunctions", default)]
    allocated_functions: Vec<ElementRef>
});

// --- Physical Link (Câble/Bus/Ondes) ---
arcadia_element!(PhysicalLink {
    #[serde(rename = "linkType", default)]
    link_type: String, // "Ethernet", "Bus", etc.

    source: ElementRef,
    target: ElementRef,

    #[serde(default)]
    transports: Vec<ElementRef> // ComponentExchanges
});

// --- Component Exchange (Physique) ---
arcadia_element!(ComponentExchange {
    source: ElementRef,
    target: ElementRef,

    #[serde(rename = "allocatedToPhysicalLink", default)]
    allocated_to_physical_link: Vec<ElementRef>,

    #[serde(rename = "allocatesFunctionalExchanges", default)]
    allocates_functional_exchanges: Vec<ElementRef>
});
