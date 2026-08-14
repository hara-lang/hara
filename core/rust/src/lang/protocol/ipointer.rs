/// Identifies the evaluator context that owns pointer resolution.
///
/// Pointer descriptor fields are accessed through the ordinary collection
/// protocols; they are deliberately not duplicated here.
pub trait IPointer<C> {
    fn pointer_context(&self) -> C;
}
