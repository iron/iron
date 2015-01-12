/// Some implementations for chains of tuples.
///
/// FIXME(reem): Move generation of this to a build script.

use super::Modifier;

impl<X, M1> Modifier<X> for (M1,)
where M1: Modifier<X> {
    fn modify(self, x: &mut X) {
        self.0.modify(x);
    }
}

impl<X, M1, M2> Modifier<X> for (M1, M2)
where M1: Modifier<X>,
      M2: Modifier<X> {
    fn modify(self, x: &mut X) {
        self.0.modify(x);
        self.1.modify(x);
    }
}

impl<X, M1, M2, M3> Modifier<X> for (M1, M2, M3)
where M1: Modifier<X>,
      M2: Modifier<X>,
      M3: Modifier<X> {
    fn modify(self, x: &mut X) {
        self.0.modify(x);
        self.1.modify(x);
        self.2.modify(x);
    }
}

impl<X, M1, M2, M3, M4> Modifier<X> for (M1, M2, M3, M4)
where M1: Modifier<X>,
      M2: Modifier<X>,
      M3: Modifier<X>,
      M4: Modifier<X> {
    fn modify(self, x: &mut X) {
        self.0.modify(x);
        self.1.modify(x);
        self.2.modify(x);
        self.3.modify(x);
    }
}

impl<X, M1, M2, M3, M4, M5> Modifier<X> for (M1, M2, M3, M4, M5)
where M1: Modifier<X>,
      M2: Modifier<X>,
      M3: Modifier<X>,
      M4: Modifier<X>,
      M5: Modifier<X> {
    fn modify(self, x: &mut X) {
        self.0.modify(x);
        self.1.modify(x);
        self.2.modify(x);
        self.3.modify(x);
        self.4.modify(x);
    }
}

impl<X, M1, M2, M3, M4, M5, M6> Modifier<X> for (M1, M2, M3, M4, M5, M6)
where M1: Modifier<X>,
      M2: Modifier<X>,
      M3: Modifier<X>,
      M4: Modifier<X>,
      M5: Modifier<X>,
      M6: Modifier<X> {
    fn modify(self, x: &mut X) {
        self.0.modify(x);
        self.1.modify(x);
        self.2.modify(x);
        self.3.modify(x);
        self.4.modify(x);
        self.5.modify(x);
    }
}
