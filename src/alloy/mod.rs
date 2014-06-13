use anymap::AnyMap;

pub struct Alloy {
    map: AnyMap
}

impl Alloy {
    pub fn new() -> Alloy {
        Alloy {
            map: AnyMap::new()
        }
    }
}

impl Alloy {
    pub fn find<'a, T: 'static>(&'a self) -> Option<&'a T> {
        self.map.find::<T>()
    }

    pub fn find_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut T> {
        self.map.find_mut::<T>()
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.map.insert::<T>(value)
    }

    pub fn remove<T: 'static>(&mut self) {
        self.remove::<T>()
    }
}

