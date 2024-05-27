macro_rules! new_option {
    ($name:ident, $default:expr, $min:expr, $max:expr) => {
        pub struct $name;

        impl $name {
            pub const DEFAULT: u32 = $default;
            pub const MIN: u32 = $min;
            pub const MAX: u32 = $max;
        }
    };
}

new_option!(Overhead, 25, 1, 1000);
new_option!(HashMb, 25, 1, 8192);
new_option!(Threads, 1, 1, 128);
