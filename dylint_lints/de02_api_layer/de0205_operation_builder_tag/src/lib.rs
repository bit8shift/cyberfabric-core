#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;

use clippy_utils::consts::{ConstEvalCtxt, Constant};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// DE0205: Operation builder must have a properly formatted tag
    ///
    /// Ensures that all `OperationBuilder` instances call `.tag(...)` with a string
    /// containing whitespace-separated words, where each word starts with a capital letter.
    /// Tags must be string literals or references to `const` string items.
    ///
    /// ### Why is this bad?
    ///
    /// Operation builders without tags or with improperly formatted tags make it difficult
    /// to organize and categorize API endpoints in OpenAPI documentation and UI.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Invalid tag format - will trigger the lint
    /// OperationBuilder::post("/users")
    ///     .operation_id("create_user")
    ///     .tag("simple resource registry");
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Proper tag format - will not trigger the lint
    /// OperationBuilder::post("/users")
    ///     .operation_id("create_user")
    ///     .tag("Simple Resource Registry");
    /// ```
    pub DE0205_OPERATION_BUILDER_TAG,
    Deny,
    "operation builder must have a tag with proper format (DE0205)"
}

impl<'tcx> LateLintPass<'tcx> for De0205OperationBuilderTag {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx rustc_hir::Expr<'tcx>) {
        // Look for method calls on OperationBuilder instances
        if let rustc_hir::ExprKind::MethodCall(path, receiver, args, _span) = expr.kind {
            // Check if this is a method call on an OperationBuilder
            if is_operation_builder_method(cx, receiver) {
                let method_name = path.ident.name.as_str();

                // If we see .tag(), validate the format
                if method_name == "tag" {
                    if let Some(tag_arg) = args.first() {
                        if let Some(tag_string) = extract_tag_value(cx, tag_arg) {
                            if !is_valid_tag_format(&tag_string) {
                                cx.span_lint(DE0205_OPERATION_BUILDER_TAG, tag_arg.span, |diag| {
                                    diag.primary_message("tag format is invalid");
                                    diag.help("tags must contain whitespace-separated words, each starting with a capital letter");
                                    diag.note("example: \"User Management\", \"Simple Resource Registry\"");
                                });
                            }
                        } else {
                            cx.span_lint(DE0205_OPERATION_BUILDER_TAG, tag_arg.span, |diag| {
                                diag.primary_message("tag must be a string literal or const string");
                                diag.help("use a string literal like `.tag(\"Your Tag\")` or a const string");
                            });
                        }
                    }
                }
            }
        }
    }
}

fn is_operation_builder_method(cx: &LateContext<'_>, expr: &rustc_hir::Expr<'_>) -> bool {
    // Check if the expression resolves to OperationBuilder type
    let ty = cx.typeck_results().expr_ty(expr);
    let type_str = format!("{:?}", ty);

    type_str.contains("OperationBuilder")
}

fn extract_tag_value(cx: &LateContext<'_>, expr: &rustc_hir::Expr<'_>) -> Option<String> {
    if let rustc_hir::ExprKind::Lit(lit) = expr.kind {
        if let rustc_ast::LitKind::Str(symbol, _) = lit.node {
            return Some(symbol.to_string());
        }
    }

    if let Some(Constant::Str(s)) = ConstEvalCtxt::new(cx).eval(expr) {
        return Some(s);
    }

    None
}

fn is_valid_tag_format(tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }

    // Split by whitespace and check each word
    let words: Vec<&str> = tag.split_whitespace().collect();

    // Must have at least one word
    if words.is_empty() {
        return false;
    }

    // Each word must start with a capital letter
    for word in words {
        if word.is_empty() || !word.chars().next().unwrap().is_uppercase() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui_examples() {
        dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_comment_annotations_match_stderr() {
        let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui");
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE0205", "Operation builder tag");
    }
}
