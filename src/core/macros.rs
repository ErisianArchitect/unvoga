
/// ```no_run
/// macro_rules! num_impl {
///     ($type:ty) => {
///         // impl here
///     };
/// }
/// for_each_int_type!(num_impl)
/// // or
/// for_each_int_type!(num_impl;signed)
/// // or
/// for_each_int_type!(num_impl;unsigned)
/// ```
#[macro_export]
macro_rules! for_each_int_type {
    ($macro:path) => {
        $crate::for_each_int_type!($macro;unsigned);
        $crate::for_each_int_type!($macro;signed);
    };
    ($macro:path;unsigned) => {
        $macro!{usize}
        $macro!{u128}
        $macro!{u64}
        $macro!{u32}
        $macro!{u16}
        $macro!{u8}
    };
    ($macro:path;signed) => {
        $macro!{isize}
        $macro!{i128}
        $macro!{i64}
        $macro!{i32}
        $macro!{i16}
        $macro!{i8}
    }
}