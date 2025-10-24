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

//! Visitors from functions or closures.

use core::any::Any;
use core::marker::PhantomData;

use crate::Visitor;
use crate::VisitorMut;

/// Type returned by `make_visitor` factories.
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

/// Create a visitor that only visits items of a specific type from a function or a closure.
pub fn make_visitor<T, F1: FnMut(&T), F2: FnMut(&T)>(enter: F1, leave: F2) -> FnVisitor<T, F1, F2> {
    FnVisitor {
        enter,
        leave,
        m: PhantomData,
    }
}

/// Create a visitor that only visits mutable items of a specific type from a function or a closure.
pub fn make_visitor_mut<T, F1: FnMut(&mut T), F2: FnMut(&mut T)>(
    enter: F1,
    leave: F2,
) -> FnVisitor<T, F1, F2> {
    FnVisitor {
        enter,
        leave,
        m: PhantomData,
    }
}
