use crate::arcadia_element;
use crate::model_engine::common::{ElementRef, I18nString};

// --- Operational Actor ---
arcadia_element!(OperationalActor {
    #[serde(rename = "isHuman", default)]
    is_human: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    organization: Option<ElementRef>,

    #[serde(rename = "allocatedActivities", default)]
    allocated_activities: Vec<ElementRef>
});

// --- Operational Entity ---
arcadia_element!(OperationalEntity {
    #[serde(default)]
    composition: Vec<ElementRef>,

    #[serde(rename = "allocatedActivities", default)]
    allocated_activities: Vec<ElementRef>
});

// --- Operational Activity ---
arcadia_element!(OperationalActivity {
    #[serde(default)]
    inputs: Vec<ElementRef>,

    #[serde(default)]
    outputs: Vec<ElementRef>,

    #[serde(rename = "allocatedTo", default)]
    allocated_to: Vec<ElementRef>
});

// --- Operational Capability ---
arcadia_element!(OperationalCapability {
    #[serde(rename = "involvedActivities", default)]
    involved_activities: Vec<ElementRef>,

    #[serde(rename = "involvedActors", default)]
    involved_actors: Vec<ElementRef>, // ou stakeholders

    #[serde(default)]
    scenarios: Vec<ElementRef>
});

// --- Operational Exchange ---
arcadia_element!(OperationalExchange {
    source: ElementRef,
    target: ElementRef,

    #[serde(rename = "exchangeItems", default)]
    exchange_items: Vec<I18nString>, // ou Ref vers ExchangeItem plus tard

    #[serde(rename = "flowType", default)]
    flow_type: String
});
