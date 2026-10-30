// `is_as_expression` is public like the other generated `is_*_expression` predicates.

use tsrs_ast::Node;
use tsrs_core::P;

#[test]
fn is_as_expression_is_public() {
    let _: fn(P<Node>) -> bool = tsrs_ast::is_as_expression;
}
