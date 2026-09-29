//! Unit tests for the phase-gated workflow transport.

use super::*;

fn tool(name: &str, write: bool) -> ToolDef {
    ToolDef { name: name.into(), description: String::new(), requires_write: write, input_schema: json!({}) }
}

#[test]
fn phase_of_maps_families() {
    assert_eq!(phase_of("product_product_new"), Phase::What);
    assert_eq!(phase_of("product_domain_new"), Phase::What);
    assert_eq!(phase_of("product_decider_validate"), Phase::What);
    assert_eq!(phase_of("product_scope_add"), Phase::What);
    assert_eq!(phase_of("product_what_declare"), Phase::What);
    assert_eq!(phase_of("product_scope_enforce"), Phase::What);
    assert_eq!(phase_of("product_how_show"), Phase::How);
    assert_eq!(phase_of("product_how_add"), Phase::How);
    assert_eq!(phase_of("product_blueprint_init"), Phase::How);
    assert_eq!(phase_of("product_design_system_bind"), Phase::How);
    assert_eq!(phase_of("product_design_system_show"), Phase::How);
    assert_eq!(phase_of("product_cell_dispatch"), Phase::How);
    assert_eq!(phase_of("product_work_unit_init"), Phase::How);
    assert_eq!(phase_of("product_work_unit_show"), Phase::How);
    assert_eq!(phase_of("product_feature_new"), Phase::Build);
    assert_eq!(phase_of("product_build_run"), Phase::Build);
    // Codegen is realisation — home phase Build (reads stay visible later).
    assert_eq!(phase_of("product_codegen_manifest"), Phase::Build);
    assert_eq!(phase_of("product_codegen_emit"), Phase::Build);
    // Back-compat: the pre-v1.9.1 `product_reify_*` names still gate to Build.
    assert_eq!(phase_of("product_reify_emit"), Phase::Build);
}

#[test]
fn codegen_tools_are_registered_with_the_right_write_gating() {
    let tools = crate::tools::build_tool_list();
    let find = |n: &str| tools.iter().find(|t| t.name == n).unwrap_or_else(|| panic!("{n} missing"));
    assert!(!find("product_codegen_backends").requires_write);
    assert!(!find("product_codegen_manifest").requires_write);
    assert!(!find("product_codegen_check").requires_write);
    assert!(find("product_codegen_emit").requires_write, "emit writes the repo");
}

#[test]
fn write_tools_lock_to_their_home_phase() {
    let new = tool("product_domain_new", true);
    let show = tool("product_domain_show", false);
    // In What, both What tools are visible.
    assert!(is_visible(&new, Phase::What));
    assert!(is_visible(&show, Phase::What));
    // In How, the What read stays but the What write is locked.
    assert!(is_visible(&show, Phase::How));
    assert!(!is_visible(&new, Phase::How));
}

#[test]
fn how_authoring_writes_live_only_in_how() {
    let add = tool("product_how_add", true);
    // Hidden while still authoring the What…
    assert!(!is_visible(&add, Phase::What));
    // …live in How…
    assert!(is_visible(&add, Phase::How));
    // …and frozen once the architecture is set and Build begins.
    assert!(!is_visible(&add, Phase::Build));
}

#[test]
fn later_phase_tools_hidden_earlier() {
    let build = tool("product_build_run", true);
    assert!(!is_visible(&build, Phase::What));
    assert!(!is_visible(&build, Phase::How));
    assert!(is_visible(&build, Phase::Build));
}

#[test]
fn question_tools_are_visible_in_every_phase() {
    let new = tool("product_question_new", true);
    let list = tool("product_question_list", false);
    for phase in Phase::all() {
        assert!(is_visible(&new, phase), "question writes reachable in {phase}");
        assert!(is_visible(&list, phase), "question reads reachable in {phase}");
    }
}

#[test]
fn question_and_how_writes_are_stamped_with_the_session() {
    let mut args = json!({"statement": "?"});
    questions::stamp_session("product_question_new", &mut args, "s-1");
    assert_eq!(args["session_id"], "s-1");
    let mut how = json!({"element": "decision"});
    questions::stamp_session("product_how_add", &mut how, "s-1");
    assert_eq!(how["session_id"], "s-1");
    let mut other = json!({});
    questions::stamp_session("product_domain_new", &mut other, "s-1");
    assert!(other.get("session_id").is_none());
}
