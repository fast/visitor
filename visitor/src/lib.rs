// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A visitor pattern implementation for traversing data structures.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

/// A visitor that can be used to traverse a data structure.
pub trait Visitor {
    /// Called when entering a node.
    ///
    /// Default implementation does nothing.
    fn enter(&mut self, this: &dyn core::any::Any) {
        let _ = this;
    }

    /// Called when exiting a node.
    ///
    /// Default implementation does nothing.
    fn exit(&mut self, this: &dyn core::any::Any) {
        let _ = this;
    }

    /// Called when entering a mutable node.
    ///
    /// Default implementation calls [`enter`](Visitor::enter).
    fn enter_mut(&mut self, this: &mut dyn core::any::Any) {
        self.enter(this);
    }

    /// Called when exiting a mutable node.
    ///
    /// Default implementation calls [`exit`](Visitor::exit).
    fn exit_mut(&mut self, this: &mut dyn core::any::Any) {
        self.exit(this);
    }
}

/// A trait for types that can be traversed by a visitor.
pub trait Traversable: core::any::Any {
    /// Traverse the data structure with the given visitor.
    fn traverse<V: Visitor>(&self, visitor: &mut V);

    /// Traverse the mutable data structure with the given visitor.
    fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V);
}

mod impl_trivial {
    use super::*;

    #[cfg(not(feature = "traverse-trivial"))]
    macro_rules! trivial_impl {
        ( $type:ty ) => {
            impl Traversable for $type {
                fn traverse<V: Visitor>(&self, _visitor: &mut V) {}
                fn traverse_mut<V: Visitor>(&mut self, _visitor: &mut V) {}
            }
        };
    }

    #[cfg(feature = "traverse-trivial")]
    macro_rules! trivial_impl {
        ( $type:ty ) => {
            impl Traversable for $type {
                fn traverse<V: Visitor>(&self, visitor: &mut V) {
                    visitor.enter(self);
                    visitor.exit(self);
                }

                fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
                    visitor.enter_mut(self);
                    visitor.exit_mut(self);
                }
            }
        };
    }

    trivial_impl!(());

    trivial_impl!(u8);
    trivial_impl!(u16);
    trivial_impl!(u32);
    trivial_impl!(u64);
    trivial_impl!(u128);
    trivial_impl!(usize);

    trivial_impl!(i8);
    trivial_impl!(i16);
    trivial_impl!(i32);
    trivial_impl!(i64);
    trivial_impl!(i128);
    trivial_impl!(isize);

    trivial_impl!(f32);
    trivial_impl!(f64);

    trivial_impl!(char);
    trivial_impl!(bool);
}

#[cfg(feature = "std")]
mod impl_std_primary {
    use super::*;
    use std::string::String;

    #[cfg(not(feature = "traverse-std"))]
    impl Traversable for String {
        fn traverse<V: Visitor>(&self, _visitor: &mut V) {}
        fn traverse_mut<V: Visitor>(&mut self, _visitor: &mut V) {}
    }

    #[cfg(feature = "traverse-std")]
    impl Traversable for String {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            visitor.enter(self);
            visitor.exit(self);
        }

        fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
            visitor.enter_mut(self);
            visitor.exit_mut(self);
        }
    }
}

#[cfg(feature = "std")]
mod impl_std_container {
    use super::*;
    use std::boxed::Box;
    use std::sync::Arc;

    // Helper traits to the generic `IntoIterator` Traversable impl
    trait DerefAndTraverse {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V);
    }

    trait DerefAndTraverseMut {
        fn deref_and_traverse_mut<V: Visitor>(self, visitor: &mut V);
    }

    // Most collections iterate over item references, this is the trait impl that handles that case
    impl<T: Traversable> DerefAndTraverse for &T {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) {
            self.traverse(visitor);
        }
    }

    impl<T: Traversable> DerefAndTraverseMut for &mut T {
        fn deref_and_traverse_mut<V: Visitor>(self, visitor: &mut V) {
            self.traverse_mut(visitor);
        }
    }

    // Map-like collections iterate over item references pairs
    impl<TK: Traversable, TV: Traversable> DerefAndTraverse for (&TK, &TV) {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) {
            self.0.traverse(visitor);
            self.1.traverse(visitor);
        }
    }

    // Map-like collections have mutable iterators that allow mutating only the value, not the key
    impl<TK, TV: Traversable> DerefAndTraverseMut for (TK, &mut TV) {
        fn deref_and_traverse_mut<V: Visitor>(self, visitor: &mut V) {
            self.1.traverse_mut(visitor);
        }
    }

    // Implement Traversal for container types in standard library.
    macro_rules! impl_drive_for_into_iterator {
        ( $type:ty ; $($generics:tt)+ ) => {
            impl< $($generics)+ > Traversable for $type
            where
                $type: 'static,
                for<'a> &'a $type: IntoIterator,
                for<'a> <&'a $type as IntoIterator>::Item: DerefAndTraverse,
                for<'a> &'a mut $type: IntoIterator,
                for<'a> <&'a mut $type as IntoIterator>::Item: DerefAndTraverseMut,
            {
                fn traverse<V: Visitor>(&self, visitor: &mut V) {
                    for item in self {
                        item.deref_and_traverse(visitor);
                    }
                }

                fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
                    for item in self {
                        item.deref_and_traverse_mut(visitor);
                    }
                }
            }
        };
    }

    impl_drive_for_into_iterator! { [T] ; T }
    impl_drive_for_into_iterator! { [T; N] ; T, const N: usize }
    impl_drive_for_into_iterator! { std::vec::Vec<T> ; T }
    impl_drive_for_into_iterator! { std::collections::BTreeSet<T> ; T }
    impl_drive_for_into_iterator! { std::collections::BinaryHeap<T> ; T }
    impl_drive_for_into_iterator! { std::collections::HashSet<T> ; T }
    impl_drive_for_into_iterator! { std::collections::LinkedList<T> ; T }
    impl_drive_for_into_iterator! { std::collections::VecDeque<T> ; T }
    impl_drive_for_into_iterator! { std::collections::BTreeMap<T, U> ; T, U }
    impl_drive_for_into_iterator! { std::collections::HashMap<T, U> ; T, U }

    impl<T: Traversable> Traversable for Option<T>
    where
        Option<T>: 'static,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            if let Some(value) = self {
                value.traverse(visitor);
            }
        }

        fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
            if let Some(value) = self {
                value.traverse_mut(visitor);
            }
        }
    }

    impl<T: Traversable, U> Traversable for Result<T, U>
    where
        Result<T, U>: 'static,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            if let Ok(value) = self {
                value.traverse(visitor);
            }
        }

        fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
            if let Ok(value) = self {
                value.traverse_mut(visitor);
            }
        }
    }

    impl<T: Traversable> Traversable for Box<T> {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            (**self).traverse(visitor);
        }

        fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
            (**self).traverse_mut(visitor);
        }
    }

    impl<T: Traversable> Traversable for Arc<T> {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            (**self).traverse(visitor);
        }

        fn traverse_mut<V: Visitor>(&mut self, visitor: &mut V) {
            (**self).traverse_mut(visitor);
        }
    }
}
