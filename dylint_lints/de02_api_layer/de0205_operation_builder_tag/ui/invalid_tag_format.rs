// simulated_dir=modules/simple-resource-registry/simple-resource-registry/src/api/rest

use modkit::api::OperationBuilder;

const INVALID_TAG: &str = "simple resource registry";

fn invalid_tag_formats() {
    let router1: OperationBuilder<_, _, ()> = OperationBuilder::post("/resources")
        .operation_id("create_resource");
    // Should trigger DE0205 - Operation builder tag
    let _ = router1.tag("simple resource registry");  // lowercase words

    let router2: OperationBuilder<_, _, ()> = OperationBuilder::get("/resources/{id}")
        .operation_id("get_resource");
    // Should trigger DE0205 - Operation builder tag
    let _ = router2.tag("Simple resource registry");  // mixed case

    let router3: OperationBuilder<_, _, ()> = OperationBuilder::put("/resources/{id}")
        .operation_id("update_resource");
    // Should trigger DE0205 - Operation builder tag
    let _ = router3.tag("registry");  // single lowercase word

    let router4: OperationBuilder<_, _, ()> = OperationBuilder::delete("/resources/{id}")
        .operation_id("delete_resource");
    // Should trigger DE0205 - Operation builder tag
    let _ = router4.tag("");  // empty string

    let tag_name = "Dynamic Tag";
    let router5: OperationBuilder<_, _, ()> = OperationBuilder::get("/resources")
        .operation_id("list_resources");
    // Should trigger DE0205 - Operation builder tag
    let _ = router5.tag(tag_name);  // variable, not string literal or const

    let router6: OperationBuilder<_, _, ()> = OperationBuilder::patch("/resources/{id}")
        .operation_id("patch_resource");
    // Should trigger DE0205 - Operation builder tag
    let _ = router6.tag(INVALID_TAG);  // const with invalid format
}

fn main() {
    invalid_tag_formats();
}
