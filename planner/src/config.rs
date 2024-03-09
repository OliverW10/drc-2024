

pub mod colours {
    use opencv::core::VecN;

    use crate::vision::ColourRange;

    // TODO: use config file
    const fn c<T>(a: T, b: T, c: T) -> VecN<T, 3> {
        VecN::<T, 3> { 0: [a, b, c]}
    }
    const fn r(l: VecN<u8, 3>, h: VecN<u8, 3>) -> ColourRange {
        ColourRange { low: l, high: h}
    }
    pub const YELLOW_MASK: ColourRange = r(c(16, 15, 90), c(45, 255, 255));
    pub const BLUE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const BLACK_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const PURPLE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const RED_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
}
