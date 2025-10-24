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

use core::any::Any;
use core::marker::PhantomData;

use crate::Visitor;
use crate::VisitorMut;

/// Type returned by [`visitor_fn`].
pub struct FnVisitor<T, F1, F2> {
    enter: F1,
    leave: F2,
    m: PhantomData<T>,
}

impl<T: Any, F1: FnMut(&T), F2: FnMut(&T)> Visitor for FnVisitor<T, F1, F2> {
    fn enter(&mut self, this: &dyn Any) {
        if let Some(item) = this.downcast_ref::<T>() {
            (self.enter)(item);
        }
    }

    fn leave(&mut self, this: &dyn Any) {
        if let Some(item) = this.downcast_ref::<T>() {
            (self.leave)(item);
        }
    }
}

impl<T: Any, F1: FnMut(&mut T), F2: FnMut(&mut T)> VisitorMut for FnVisitor<T, F1, F2> {
    fn enter_mut(&mut self, this: &mut dyn Any) {
        if let Some(item) = this.downcast_mut::<T>() {
            (self.enter)(item);
        }
    }

    fn leave_mut(&mut self, this: &mut dyn Any) {
        if let Some(item) = this.downcast_mut::<T>() {
            (self.leave)(item);
        }
    }
}

/// Create a visitor that only visits items of some specific type from a function or a closure.
pub fn visitor_fn<T, F1: FnMut(&T), F2: FnMut(&T)>(enter: F1, leave: F2) -> FnVisitor<T, F1, F2> {
    FnVisitor {
        enter,
        leave,
        m: PhantomData,
    }
}

/// Similar to [`visitor_fn`], but the closure will only be called on entering the node.
pub fn visitor_enter_fn<T, F: FnMut(&T)>(enter: F) -> FnVisitor<T, F, fn(&T)> {
    visitor_fn(enter, |_| {})
}

/// Similar to [`visitor_fn`], but the closure will only be called on leaving the node.
pub fn visitor_leave_fn<T, F: FnMut(&T)>(leave: F) -> FnVisitor<T, fn(&T), F> {
    visitor_fn(|_| {}, leave)
}

/// Create a visitor that only visits mutable items of some specific type from a function or a
/// closure.
pub fn visitor_fn_mut<T, F1: FnMut(&mut T), F2: FnMut(&mut T)>(
    enter: F1,
    leave: F2,
) -> FnVisitor<T, F1, F2> {
    FnVisitor {
        enter,
        leave,
        m: PhantomData,
    }
}

/// Similar to [`visitor_fn_mut`], but the closure will only be called on entering the node.
pub fn visitor_enter_fn_mut<T, F: FnMut(&mut T)>(enter: F) -> FnVisitor<T, F, fn(&mut T)> {
    visitor_fn_mut(enter, |_| {})
}

/// Similar to [`visitor_fn_mut`], but the closure will only be called on leaving the node.
pub fn visitor_leave_fn_mut<T, F: FnMut(&mut T)>(leave: F) -> FnVisitor<T, fn(&mut T), F> {
    visitor_fn_mut(|_| {}, leave)
}
