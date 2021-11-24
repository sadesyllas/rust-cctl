macro_rules! try_downcast_ref {
    ($m:ident, $t:ty) => {
        $m.as_any().downcast_ref::<$t>()
    };
}

pub(crate) use try_downcast_ref;
