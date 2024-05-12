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
    pub const YELLOW_MASK: ColourRange = r(c(0, 0, 222), c(179, 255, 255));
    pub const BLUE_MASK: ColourRange = r(c(110, 90, 120), c(120, 255, 255));
    // pub const BLACK_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    // pub const PURPLE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    // pub const RED_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
}

pub mod plan {
    pub const PLAN_STEP_SIZE_METERS: f64 = 0.2;
    pub const PLAN_MAX_LENGTH_METERS: f64 = 4.0;
    pub const PLAN_MAX_STEPS: u32 = (PLAN_MAX_LENGTH_METERS / PLAN_STEP_SIZE_METERS) as u32;

    pub const MAX_CURVATURE: f64 = 1.0 / 0.3;
}
