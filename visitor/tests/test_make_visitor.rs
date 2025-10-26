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

#![cfg(feature = "traverse-trivial")]

use core::ops::ControlFlow;

use visitor::Traversable;
use visitor::TraversableMut;

#[test]
fn test_make_visitor() {
    let mut visitor = visitor::function::make_visitor_enter::<i32, (), _>(|item| {
        assert_eq!(*item, 42);
        ControlFlow::Continue(())
    });

    let result = 42i32.traverse(&mut visitor);
    assert!(result.is_continue());
}

#[test]
fn test_make_visitor_mut() {
    let mut visitor = visitor::function::make_visitor_leave_mut::<i32, (), _>(|item| {
        *item += 1;
        ControlFlow::Continue(())
    });

    let mut data = 42i32;
    let result = data.traverse_mut(&mut visitor);
    assert!(result.is_continue());
    assert_eq!(data, 43);
}
