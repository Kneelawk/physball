#[macro_export]
macro_rules! type_expr {
    ($ty:ty, $expr:expr) => {{
        let e: $ty = $expr;
        e
    }};
}
