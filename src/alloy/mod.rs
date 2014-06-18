//! Exposes the `Alloy` type, a flexible storage container
//! that `Middleware` may use to expose new APIs or public data
//! to other `Middleware`.

use anymap::AnyMap;

/// `Alloy` wraps an AnyMap, a map allowing storage keyed by type
/// to allow for persistent data across `Middleware`.
///
/// `Middleware` can be inserted into an `Alloy` and retrieved later. Data needing
/// exposure across `Middleware` and persistence (for example, a body parser's parsed data)
/// should be stored and retrieved later from the `Alloy`.
///
/// Only one instance of any type can be stored in the `Alloy` at a time.
/// Best practice is to store targeted data. For example, for a body parser,
/// rather than store the `Middleware`, store a `Parsed` type:
///
/// ```ignore
/// impl<Rq, Rs> Middleware for BodyParser {
///     fn enter(req: &mut Rq, res: &mut Rs, alloy: &mut Alloy) -> Status {
///         let parsed: Parsed = ...; // Parse the body
///         alloy.insert::<Parsed>(Parsed);
///     }
/// }
/// ```
///
/// In most cases, the `Middleware` itself does not need to be exposed,
/// and should not be stored on the `Alloy`.
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

