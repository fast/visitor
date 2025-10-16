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

#[cfg(feature = "derive")]
/// See [`Traversable`].
pub use visitor_derive::Traversable;
#[cfg(feature = "derive")]
/// See [`TraversableMut`].
pub use visitor_derive::TraversableMut;

pub use self::impl_visitor::*;

/// Whether the visitor is entering or exiting a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The visitor is entering a node.
    Enter,
    /// The visitor is exiting a node.
    Exit,
}

/// A visitor that can be used to traverse a data structure.
pub trait Visitor {
    /// Called when the visitor is traversing a node.
    fn visit(&mut self, this: &dyn core::any::Any, event: Event);
}

/// A visitor that can be used to traverse a mutable data structure.
pub trait VisitorMut {
    /// Called when the visitor is traversing a mutable node.
    fn visit_mut(&mut self, this: &mut dyn core::any::Any, event: Event);
}

mod impl_visitor {
    use core::any::Any;
    use core::marker::PhantomData;

    use super::*;

    /// Type returned by [`visitor_fn`].
    pub struct FnVisitor<T, F> {
        f: F,
        m: PhantomData<T>,
    }

    impl<T: Any, F: FnMut(&T, Event)> Visitor for FnVisitor<T, F> {
        fn visit(&mut self, this: &dyn Any, event: Event) {
            if let Some(item) = <dyn Any>::downcast_ref::<T>(this) {
                (self.f)(item, event);
            }
        }
    }

    impl<T: Any, F: FnMut(&mut T, Event)> VisitorMut for FnVisitor<T, F> {
        fn visit_mut(&mut self, this: &mut dyn Any, event: Event) {
            if let Some(item) = <dyn Any>::downcast_mut::<T>(this) {
                (self.f)(item, event);
            }
        }
    }

    /// Create a visitor that only visits items of some specific type from a function or a closure.
    pub fn visitor_fn<T, F: FnMut(&T, Event)>(f: F) -> FnVisitor<T, F> {
        FnVisitor { f, m: PhantomData }
    }

    /// Similar to [`visitor_fn`], but the closure will only be called on [`Event::Enter`].
    pub fn visitor_enter_fn<T, F: FnMut(&T)>(mut f: F) -> FnVisitor<T, impl FnMut(&T, Event)> {
        visitor_fn(move |item, event| {
            if let Event::Enter = event {
                f(item);
            }
        })
    }

    /// Similar to [`visitor_fn`], but the closure will only be called on [`Event::Exit`].
    pub fn visitor_exit_fn<T, F: FnMut(&T)>(mut f: F) -> FnVisitor<T, impl FnMut(&T, Event)> {
        visitor_fn(move |item, event| {
            if let Event::Exit = event {
                f(item);
            }
        })
    }

    /// Create a visitor that only visits mutable items of some specific type from a function or a
    /// closure.
    pub fn visitor_fn_mut<T, F: FnMut(&mut T, Event)>(f: F) -> FnVisitor<T, F> {
        FnVisitor { f, m: PhantomData }
    }

    /// Similar to [`visitor_fn_mut`], but the closure will only be called on [`Event::Enter`].
    pub fn visitor_enter_fn_mut<T, F: FnMut(&mut T)>(
        mut f: F,
    ) -> FnVisitor<T, impl FnMut(&mut T, Event)> {
        visitor_fn_mut(move |item, event| {
            if let Event::Enter = event {
                f(item);
            }
        })
    }

    /// Similar to [`visitor_fn_mut`], but the closure will only be called on [`Event::Exit`].
    pub fn visitor_exit_fn_mut<T, F: FnMut(&mut T)>(
        mut f: F,
    ) -> FnVisitor<T, impl FnMut(&mut T, Event)> {
        visitor_fn_mut(move |item, event| {
            if let Event::Exit = event {
                f(item);
            }
        })
    }
}

/// A trait for types that can be traversed by a visitor.
pub trait Traversable: core::any::Any {
    /// Traverse the data structure with the given visitor.
    fn traverse<V: Visitor>(&self, visitor: &mut V);
}

/// A trait for types that can be traversed mutably by a visitor.
pub trait TraversableMut: core::any::Any {
    /// Traverse the mutable data structure with the given visitor.
    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V);
}

#[allow(unused_macros)]
macro_rules! blank_traverse_impl {
    ( $type:ty ) => {
        impl Traversable for $type {
            fn traverse<V: Visitor>(&self, _visitor: &mut V) {}
        }

        impl TraversableMut for $type {
            fn traverse_mut<V: VisitorMut>(&mut self, _visitor: &mut V) {}
        }
    };
}

#[allow(unused_macros)]
macro_rules! trivial_traverse_impl {
    ( $type:ty ) => {
        impl Traversable for $type {
            fn traverse<V: Visitor>(&self, visitor: &mut V) {
                visitor.visit(self, Event::Enter);
                visitor.visit(self, Event::Exit);
            }
        }

        impl TraversableMut for $type {
            fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
                visitor.visit_mut(self, Event::Enter);
                visitor.visit_mut(self, Event::Exit);
            }
        }
    };
}

mod impl_trivial {
    use super::*;

    #[cfg(not(feature = "traverse-trivial"))]
    macro_rules! trivial_impl {
        ( $type:ty ) => {
            blank_traverse_impl!($type);
        };
    }

    #[cfg(feature = "traverse-trivial")]
    macro_rules! trivial_impl {
        ( $type:ty ) => {
            trivial_traverse_impl!($type);
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

mod impl_tuple {
    use super::*;

    macro_rules! tuple_impl {
        ( $( $( $type:ident ),+ => $( $field:tt ),+ )+ ) => {
            $(
                impl<$( $type ),+> Traversable for ($($type,)+)
                where
                    $(
                        $type: Traversable
                    ),+
                {
                    fn traverse<V: Visitor>(&self, visitor: &mut V) {
                        $(
                            self.$field.traverse(visitor);
                        )+
                    }
                }

                impl<$( $type ),+> TraversableMut for ($($type,)+)
                where
                    $(
                        $type: TraversableMut
                    ),+
                {
                    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
                        $(
                            self.$field.traverse_mut(visitor);
                        )+
                    }
                }
            )+
        };
    }

    tuple_impl! {
        T0 => 0
        T0, T1 => 0, 1
        T0, T1, T2 => 0, 1, 2
        T0, T1, T2, T3 => 0, 1, 2, 3
        T0, T1, T2, T3, T4 => 0, 1, 2, 3, 4
        T0, T1, T2, T3, T4, T5 => 0, 1, 2, 3, 4, 5
        T0, T1, T2, T3, T4, T5, T6 => 0, 1, 2, 3, 4, 5, 6
        T0, T1, T2, T3, T4, T5, T6, T7 => 0, 1, 2, 3, 4, 5, 6, 7
        T0, T1, T2, T3, T4, T5, T6, T7, T8 => 0, 1, 2, 3, 4, 5, 6, 7, 8
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
    }
}

#[cfg(feature = "std")]
mod impl_std_primary {
    use std::string::String;

    use super::*;

    #[cfg(not(feature = "traverse-std"))]
    macro_rules! std_primary_impl {
        ( $type:ty ) => {
            blank_traverse_impl!($type);
        };
    }

    #[cfg(feature = "traverse-std")]
    macro_rules! std_primary_impl {
        ( $type:ty ) => {
            trivial_traverse_impl!($type);
        };
    }

    std_primary_impl!(String);
}

#[cfg(feature = "std")]
mod impl_std_container {
    use std::boxed::Box;
    use std::cell::Cell;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::RwLock;

    use super::*;

    // Helper traits to the generic `IntoIterator` Traversable impl
    trait DerefAndTraverse {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V);
    }

    trait DerefAndTraverseMut {
        fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V);
    }

    // Most collections iterate over item references, this is the trait impl that handles that case
    impl<T: Traversable> DerefAndTraverse for &T {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) {
            self.traverse(visitor);
        }
    }

    impl<T: TraversableMut> DerefAndTraverseMut for &mut T {
        fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) {
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
    impl<TK, TV: TraversableMut> DerefAndTraverseMut for (TK, &mut TV) {
        fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) {
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
            {
                #[allow(for_loops_over_fallibles)]
                fn traverse<V: Visitor>(&self, visitor: &mut V) {
                    for item in self {
                        item.deref_and_traverse(visitor);
                    }
                }
            }

            impl< $($generics)+ > TraversableMut for $type
            where
                $type: 'static,
                for<'a> &'a mut $type: IntoIterator,
                for<'a> <&'a mut $type as IntoIterator>::Item: DerefAndTraverseMut,
            {
                #[allow(for_loops_over_fallibles)]
                fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
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
    impl_drive_for_into_iterator! { Option<T> ; T }
    impl_drive_for_into_iterator! { Result<T, U> ; T, U }

    impl<T: Traversable> Traversable for Box<T> {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            (**self).traverse(visitor);
        }
    }

    impl<T: TraversableMut> TraversableMut for Box<T> {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
            (**self).traverse_mut(visitor);
        }
    }

    impl<T: Traversable> Traversable for Arc<T> {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            (**self).traverse(visitor);
        }
    }

    impl<T> Traversable for Mutex<T>
    where
        T: Traversable,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            let lock = self.lock().unwrap();
            lock.traverse(visitor);
        }
    }

    impl<T> Traversable for RwLock<T>
    where
        T: Traversable,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            let lock = self.read().unwrap();
            lock.traverse(visitor);
        }
    }

    impl<T> TraversableMut for Arc<Mutex<T>>
    where
        T: TraversableMut,
    {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
            let mut lock = self.lock().unwrap();
            lock.traverse_mut(visitor);
        }
    }

    impl<T> TraversableMut for Arc<RwLock<T>>
    where
        T: TraversableMut,
    {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
            let mut lock = self.write().unwrap();
            lock.traverse_mut(visitor);
        }
    }

    impl<T> Traversable for Cell<T>
    where
        T: Traversable + Copy,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) {
            self.get().traverse(visitor);
        }
    }

    impl<T> TraversableMut for Cell<T>
    where
        T: TraversableMut,
    {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
            self.get_mut().traverse_mut(visitor);
        }
    }
}
