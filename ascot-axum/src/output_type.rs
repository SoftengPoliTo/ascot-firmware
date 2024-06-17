use core::any::type_name;
use core::future::Future;

use crate::device::{DeviceError, DevicePayload};

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

pub(crate) mod private {
    pub trait OutputTypeName<Args> {
        fn output_type_name(&self) -> &'static str;
    }
}

impl<F, Fut> private::OutputTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<DevicePayload, DeviceError>> + Send,
{
    fn output_type_name(&self) -> &'static str {
        type_name::<Fut::Output>()
    }
}

macro_rules! impl_output_type_name {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        impl<F, Fut, M, $($ty,)* $last> private::OutputTypeName<(M, $($ty,)* $last,)> for F
        where
            F: FnOnce($($ty,)* $last,) -> Fut,
            Fut: Future<Output = Result<DevicePayload, DeviceError>> + Send,
            {
                fn output_type_name(&self) -> &'static str {
                    type_name::<Fut::Output>()
                }
            }
    };
}

all_the_tuples!(impl_output_type_name);
