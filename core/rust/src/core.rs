#![allow(clippy::too_many_lines)] // Temporary compatibility facade during Java-port split.
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};

pub use crate::kernel::Form;
use crate::kernel::{NamespaceLoadState, NamespaceRegistry, Var as KernelVar, VarOrigin};
use crate::lang::data::List as PList;
use crate::lang::data::{
    Atom as PAtom, Cons as PCons, Deque as PDeque, Keyword, Map as PMap, OrderedMap as POrderedMap,
    OrderedSet as POrderedSet, Pointer as PPointer, Queue as PQueue, Set as PSet,
    PriorityMap as PPriorityMap, SortedMap as PSortedMap, SortedSet as PSortedSet, Symbol, TaggedLiteral as PTaggedLiteral,
    Trie as PTrie, Tuple as PTuple, Vector as PVector, Seq as PSeq,
};
use crate::lang::data::{Metadata, MetadataValue};
use crate::lang::data::{
    MutableList, MutableMap, MutableOrderedMap, MutableOrderedSet, MutableQueue, MutableSet,
    MutableSortedMap, MutableSortedSet, MutableTrie, MutableVector,
};
use crate::lang::hash::JavaHash;
use crate::lang::protocol::{
    IDisplay, IEmpty, IMetadata, INamespaced, IPopFirst, IPopLast, IToMutable, IToPersistent,
};
use crate::numeric::{self, ArithmeticOp};
pub use crate::task::{
    LocalPromiseProvider, Promise, PromiseProvider, PromiseRejection, PromiseState,
};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[path = "fiber.rs"]
mod fiber;
#[path = "core/native_result.rs"]
mod native_result;
pub use native_result::{ResultStatus, ResultValue};
#[path = "native_crypto.rs"]
mod native_crypto;
pub(crate) use fiber::Cont;
pub use fiber::{EvalFiber, EvalFiberState, Step};


include!("core/inventory.rs");
include!("core/value.rs");
include!("core/inspection.rs");
include!("core/environment.rs");
include!("core/native.rs");
include!("core/provider.rs");
include!("core/async_value.rs");
include!("core/primitive.rs");
include!("core/protocol.rs");
include!("core/operation.rs");
include!("core/form.rs");
include!("core/namespace.rs");
include!("core/evaluator.rs");
