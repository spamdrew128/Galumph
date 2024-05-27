macro_rules! new_option {
    ($name:ident, $default:expr, $min:expr, $max:expr, $name_str:expr) => {
        pub struct $name;

        impl $name {
            pub const DEFAULT: u32 = $default;
            pub const MIN: u32 = $min;
            pub const MAX: u32 = $max;
            pub const STR: &'static str = $name_str;
        }
    };
}

macro_rules! option_string {
    ($($opt:ident),*) => {{
        let mut res = String::new();
        $(
            res.push_str(
                format!("option name {} type spin default {} min {} max {}\n",
                $opt::STR,
                $opt::DEFAULT,
                $opt::MIN,
                $opt::MAX).as_str()
            );
        )*
        res
    }};
}

new_option!(Overhead, 25, 1, 1000, "Overhead");
new_option!(HashMb, 25, 1, 8192, "HashMb");
new_option!(Threads, 1, 1, 128, "Threads");

pub fn display_options() {
    let options = option_string!(Overhead, HashMb, Threads);
    println!("{options}");
}
