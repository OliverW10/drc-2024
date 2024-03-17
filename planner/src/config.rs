pub mod colours {
    use opencv::core::VecN;


    pub struct ColourRange {
        pub low: VecN<u8, 3>,
        pub high: VecN<u8, 3>,
    }

    // TODO: use config file
    const fn c<T>(a: T, b: T, c: T) -> VecN<T, 3> {
        VecN::<T, 3> { 0: [a, b, c] }
    }
    const fn r(l: VecN<u8, 3>, h: VecN<u8, 3>) -> ColourRange {
        ColourRange { low: l, high: h }
    }
    pub const YELLOW_MASK: ColourRange = r(c(20, 20, 120), c(45, 255, 255));
    pub const BLUE_MASK: ColourRange = r(c(110, 90, 120), c(120, 255, 255));
    pub const BLACK_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const PURPLE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const RED_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
}
