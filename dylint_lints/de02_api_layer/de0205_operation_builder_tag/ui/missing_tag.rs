// simulated_dir=modules/simple-resource-registry/simple-resource-registry/src/api/rest

use modkit::api::OperationBuilder;

fn test_operations() {
    // No tag calls here - should not trigger DE0205
    let router1: OperationBuilder<_, _, ()> = OperationBuilder::post("/resources")
        .operation_id("create_resource");

    let router2: OperationBuilder<_, _, ()> = OperationBuilder::get("/resources/{id}")
        .operation_id("get_resource");

    _ = router1;
    _ = router2;
}

fn main() {
    test_operations();
}
