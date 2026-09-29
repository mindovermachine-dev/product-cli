//! Open-question tool definitions — CLI↔MCP parity for `product question`.

use super::ToolDef;
use serde_json::json;

fn def(name: &str, description: &str, write: bool, props: serde_json::Value, required: serde_json::Value) -> ToolDef {
    let mut props = props;
    if let Some(m) = props.as_object_mut() {
        m.insert("product".into(), json!({"type": "string"}));
    }
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: write,
        input_schema: json!({"type": "object", "properties": props, "required": required}),
    }
}

/// Every `product_question_*` tool. Visible in every workflow phase.
pub(super) fn all() -> Vec<ToolDef> {
    vec![
        new_tool(),
        def("product_question_list",
            "List open questions, filtered by status / concerns (node id) / context / blocking. include_derived=true adds the recomputed validation/completeness gaps; every row carries `source: authored | derived`.",
            false,
            json!({
                "status": {"type": "string"}, "concerns": {"type": "string"}, "context": {"type": "string"},
                "blocking": {"type": "string"}, "include_derived": {"type": "boolean"}
            }),
            json!([])),
        def("product_question_show",
            "Show one question with the current state of each node it concerns.",
            false, json!({"id": {"type": "string"}}), json!(["id"])),
        def("product_question_answer",
            "Settle a question: status answered | deferred | wont-fix, a `resolution` (required), and optionally `resolved_by` — the What nodes or How decisions that answered it.",
            true,
            json!({
                "id": {"type": "string"},
                "status": {"type": "string", "description": "answered | deferred | wont-fix (default answered)"},
                "resolution": {"type": "string"},
                "resolved_by": {"type": "array", "items": {"type": "string"}}
            }),
            json!(["id"])),
        def("product_question_rm", "Delete a question (e.g. raised by mistake).",
            true, json!({"id": {"type": "string"}}), json!(["id"])),
        def("product_question_export",
            "Render the questions as a Markdown document grouped by context and status, for readers outside the tool.",
            false, json!({"format": {"type": "string", "description": "md"}}), json!([])),
    ]
}

/// `product_question_new` — raise (or promote) a question.
fn new_tool() -> ToolDef {
    let gate = json!({"type": ["string", "boolean"], "description": "The gate it blocks: what | how | build | finalize; false/empty = informational"});
    def("product_question_new",
            "Raise an open question the graph cannot yet decide, next to the nodes it concerns (a `q-*` What node). `blocking` names the gate it holds shut (what | how | build | finalize). Pass `from_derived` (the exact text of a derived question from product_question_list) to promote a recomputed gap into a persistent question. Available in every phase.",
            true,
            json!({
                "statement": {"type": "string"},
                "concerns": {"type": "array", "items": {"type": "string"}, "description": "Node ids the question is about (≥ 1)"},
                "id": {"type": "string", "description": "Default: the next free q-NNN"},
                "context": {"type": "string"},
                "blocking": gate,
                "raised_by": {"type": "string"},
                "from_derived": {"type": "string", "description": "Promote this derived question (its text)"}
            }),
            json!([]))
}
