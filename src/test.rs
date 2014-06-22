extern crate iron;
extern crate persistent;

#[cfg(test)]
mod test {
    use std::sync::{Arc, RWLock};
    use persistent::Persistent;
    use iron::{Chain, StackChain, Request, Response, Alloy};
    use std::mem::uninitialized;

    fn unsafe_dispatch<C: Chain>(chain: &mut C) {
        unsafe {
            let _ = chain.dispatch(uninitialized(), uninitialized(), None);
        }
    }

    #[test]
    fn inserts_data() {
        fn data_exists(_: &mut Request, _: &mut Response, alloy: &mut Alloy) {
            assert_eq!(*alloy.find::<Persistent<int, int>>().unwrap().data.read(), 154);
        }

        let data = Arc::new(RWLock::new(154));
        let persistent: Persistent<int, int> = Persistent { data: data.clone() };
        let mut testchain: StackChain = Chain::new();
        testchain.link(persistent);
        testchain.link(data_exists);

        unsafe_dispatch(&mut testchain);
    }

    #[test]
    fn changes_when_modified() {
        fn modify_data(_: &mut Request, _: &mut Response, alloy: &mut Alloy) {
            *alloy.find::<Persistent<int, int>>().unwrap().data.write() += 1;
        }

        fn data_modified(_: &mut Request, _: &mut Response, alloy: &mut Alloy) {
            assert_eq!(*alloy.find::<Persistent<int, int>>().unwrap().data.read(), 155);
        }

        let data = Arc::new(RWLock::new(154));
        let persistent: Persistent<int, int> = Persistent { data: data.clone() };
        let mut testchain: StackChain = Chain::new();
        testchain.link(persistent);
        testchain.link(modify_data);
        testchain.link(data_modified);

        unsafe_dispatch(&mut testchain);
    }

    #[test]
    fn persists_between_calls() {
        fn modify_data(_: &mut Request, _: &mut Response, alloy: &mut Alloy) {
            *alloy.find::<Persistent<int, int>>().unwrap().data.write() += 1;
        }

        fn data_modified(_: &mut Request, _: &mut Response, alloy: &mut Alloy) {
            assert_eq!(*alloy.find::<Persistent<int, int>>().unwrap().data.read(), 156);
        }

        let data = Arc::new(RWLock::new(154));
        let persistent: Persistent<int, int> = Persistent { data: data.clone() };
        let mut testchain: StackChain = Chain::new();
        testchain.link(persistent);
        testchain.link(modify_data);
        unsafe_dispatch(&mut testchain);

        testchain.link(data_modified);
        unsafe_dispatch(&mut testchain);
    }
}

