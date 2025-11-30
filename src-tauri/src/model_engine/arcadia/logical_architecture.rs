use crate::arcadia_element;
use crate::model_engine::common::ElementRef;

// --- Logical Component ---
arcadia_element!(LogicalComponent {
    #[serde(rename = "isAbstract", default)]
    is_abstract: bool,

    #[serde(rename = "subComponents", default)]
    sub_components: Vec<ElementRef>,

    #[serde(rename = "allocatedFunctions", default)]
    allocated_functions: Vec<ElementRef>,

    #[serde(rename = "realizedSystemComponents", default)]
    realized_system_components: Vec<ElementRef>,

    #[serde(rename = "providedInterfaces", default)]
    provided_interfaces: Vec<ElementRef>,

    #[serde(rename = "requiredInterfaces", default)]
    required_interfaces: Vec<ElementRef>
});

// --- Logical Function ---
arcadia_element!(LogicalFunction {
    #[serde(rename = "realizedSystemFunctions", default)]
    realized_system_functions: Vec<ElementRef>,

    #[serde(rename = "allocatedTo", default)]
    allocated_to: Vec<ElementRef>,

    #[serde(default)]
    inputs: Vec<ElementRef>,

    #[serde(default)]
    outputs: Vec<ElementRef>,

    #[serde(rename = "subFunctions", default)]
    sub_functions: Vec<ElementRef>
});

// --- Logical Actor ---
arcadia_element!(LogicalActor {
    #[serde(rename = "isHuman", default)]
    is_human: bool,

    #[serde(rename = "realizedSystemActors", default)]
    realized_system_actors: Vec<ElementRef>,

    #[serde(rename = "allocatedFunctions", default)]
    allocated_functions: Vec<ElementRef>
});

// --- Functional Exchange (LA) ---
arcadia_element!(FunctionalExchange {
    source: ElementRef,
    target: ElementRef,

    #[serde(rename = "exchangeItems", default)]
    exchange_items: Vec<ElementRef>,

    #[serde(rename = "realizedSystemExchanges", default)]
    realized_system_exchanges: Vec<ElementRef>,

    #[serde(rename = "allocatedToComponentExchange", default)]
    allocated_to_component_exchange: Vec<ElementRef>
});

// --- Component Exchange (Logique) ---
arcadia_element!(ComponentExchange {
    source: ElementRef,
    target: ElementRef,

    #[serde(rename = "allocatesFunctionalExchanges", default)]
    allocates_functional_exchanges: Vec<ElementRef>,

    #[serde(default)]
    orientation: String // "Unidirectional" | "Bidirectional"
});

// --- Logical Interface ---
arcadia_element!(LogicalInterface {
    #[serde(rename = "isProvidedBy", default)]
    is_provided_by: Vec<ElementRef>,

    #[serde(rename = "isRequiredBy", default)]
    is_required_by: Vec<ElementRef>,

    #[serde(rename = "exchangeItems", default)]
    exchange_items: Vec<ElementRef>
});
