//! Overloadable modification through both owned and mutable references
//! to a type with minimal code duplication.
//!
// FIXME(reem): tracking rust-lang/rust#20974
//
// Currently copied from `modifier` the crate, since cross-crate
// impls for this are broken.

/// Allows use of the implemented type as an argument to Set::set.
///
/// This allows types to be used for ad-hoc overloading of Set::set
/// to perform complex updates to the parameter of Modifier.
pub trait Modifier<F> {
    /// Modify `F` with self.
    fn modify(self, &mut F);
}

/// A trait providing the set and set_mut methods for all types.
///
/// Simply implement this for your types and they can be used
/// with modifiers.
pub trait Set: Sized {
    /// Modify self using the provided modifier.
    #[inline(always)]
    fn set<M: Modifier<Self>>(mut self, modifier: M) -> Self {
        modifier.modify(&mut self);
        self
    }

    /// Modify self through a mutable reference with the provided modifier.
    #[inline(always)]
    fn set_mut<M: Modifier<Self>>(&mut self, modifier: M) -> &mut Self {
        modifier.modify(self);
        self
    }
}

mod impls;

