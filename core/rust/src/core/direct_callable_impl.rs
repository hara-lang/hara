mod base_direct_callable_impl {
    use super::*;
    include!("direct_callable_impl_base.rs");
}

include!("direct_callable_bootstrap.rs");
include!("direct_callable_impl_overlay.rs");
