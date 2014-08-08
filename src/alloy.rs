//! Exposes the `Alloy` type, a flexible storage container
//! that `Middleware` may use to expose new APIs or public data
//! to other `Middleware`.

use anymap::AnyMap;

/// `Alloy` wraps an `AnyMap` - a map keyed by types that facilitates
/// persistent data access across `Middleware`.
///
/// Each `Request` has an `Alloy` attached to it that persists for the life
/// of the request. Middleware can use `Request::alloy` to pass data to other middleware,
/// or store data for use in their own `exit` function.
///
/// Best practice is to store targeted data. For example, a body parser
/// should store a `Parsed` type, rather than itself:
///
/// ```ignore
/// impl Middleware for BodyParser {
///     fn enter(req: &mut Request, res: &mut Response) -> Status {
///         let parsed: Parsed = ...; // Parse the body
///         req.alloy.insert::<Parsed>(parsed);
///         Continue
///     }
/// }
/// ```
///
pub struct Alloy {
    map: AnyMap
}

impl Alloy {
    /// Create a new, empty alloy.
    pub fn new() -> Alloy {
        Alloy {
            map: AnyMap::new()
        }
    }
}

impl Alloy {
    /// Get a reference to the stored value of a given type.
    pub fn find<'a, T: 'static>(&'a self) -> Option<&'a T> {
        self.map.find::<T>()
    }

    /// Get a mutable reference to the stored value of a given type.
    pub fn find_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut T> {
        self.map.find_mut::<T>()
    }

    /// Add an value to the `Alloy`.
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.map.insert::<T>(value)
    }

    /// Remove a value from the `Alloy`.
    pub fn remove<T: 'static>(&mut self) {
        self.remove::<T>()
    }
}

