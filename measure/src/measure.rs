use {
    std::{fmt, time::{Duration, Instant}},
};


#[derive(Debug)]
pub struct Measure {
    name: &'static str,
    start: Instant,
    duration: u64,
}

fn duration_as_ns(d: &Duration) -> u64 {
    d.as_secs()
        .saturating_mul(1_000_000_000)
        .saturating_add(u64::from(d.subsec_nanos()))
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::sync::Once;
    static INIT: Once = Once::new();
    static mut DUR_TEST: u64 = 0;
    static mut TICKS_TEST: u64 = 1;
    static mut TICKS_OVERHEAD: u64 = 1_000_000;
    use std::fmt;
    use std::time::{Instant, Duration};
    use super::duration_as_ns;

    pub fn check_rdtscp() -> bool {
        use std::arch;
        let result = unsafe {
                        arch::x86_64::__cpuid_count(0x80000001u32, 0) };
        (result.edx & (1 << 27)) != 0
    }

    fn rdtscp() -> u64 {
        use std::arch::asm;
        let edx: u32;
        let eax: u32;
        unsafe {
            asm!("rdtscp",
                 out("edx") edx,
                 out("eax") eax,
                 out("cx") _);
        }
        ((edx as u64) << 32) + (eax as u64)
    }

    fn calibration() {
        if ! check_rdtscp() { return }
        // calibrication overhead
        for _i in 0 .. 100 {
            let start = rdtscp();
            let stop = rdtscp();
            let diff = stop - start;
            unsafe {
                if TICKS_OVERHEAD > diff {
                    TICKS_OVERHEAD = diff
                }
            }
        }
        let t_start = Instant::now();
        let start = rdtscp();
        std::thread::sleep(Duration::from_millis(100));
        let stop = rdtscp();
        unsafe {
            DUR_TEST = duration_as_ns(& t_start.elapsed());
            TICKS_TEST = stop - start;
        }
    }
    #[allow(dead_code)]
    pub fn tsc_status() -> String {
        INIT.call_once(|| {
            calibration();
        });
        let ticks_us: f32 = unsafe {
                        TICKS_TEST as f32 / DUR_TEST as f32
        };
        format!("rdtscp ticks {:.3} per ns", ticks_us)
    }

    pub struct MeasureTsc {
        name: &'static str,
        start: u64,
        duration: u64,
    }
    impl MeasureTsc {
        pub fn start(name: &'static str) -> Self {
            INIT.call_once(|| {
                calibration();
            });
            assert!(check_rdtscp(), "rdtscp NOT SUPPORTED!");
            Self {
                name,
                start: rdtscp(),
                duration: 0,
            }
        }
        pub fn stop(&mut self) {
            self.duration = unsafe {
                (rdtscp() - self.start - TICKS_OVERHEAD)
                    .saturating_mul(DUR_TEST)
                    / TICKS_TEST
            }
        }
        pub fn as_ns(&self) -> u64 {
            self.duration
        }
        pub fn as_us(&self) -> u64 {
            self.duration / 1000
        }
        pub fn as_ms(&self) -> u64 {
            self.duration / (1000 * 1000)
        }
    }

    impl fmt::Display for MeasureTsc {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.duration == 0 {
                write!(f, "{} running", self.name)
            } else if self.as_us() < 1 {
                write!(f, "{} took {}ns", self.name, self.duration)
            } else if self.as_ms() < 1 {
                write!(f, "{} took {}us", self.name, self.as_us())
            } else {
                write!(f, "{} took {}ms", self.name, self.as_ms())
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub use x86::{check_rdtscp, MeasureTsc};

impl Measure {
    pub fn start(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            duration: 0,
        }
    }

    pub fn stop(&mut self) {
        self.duration = duration_as_ns(&self.start.elapsed());
    }

    pub fn as_ns(&self) -> u64 {
        self.duration
    }

    pub fn as_us(&self) -> u64 {
        self.duration / 1000
    }

    pub fn as_ms(&self) -> u64 {
        self.duration / (1000 * 1000)
    }

    pub fn as_s(&self) -> f64 {
        //self.duration as f64 / 1_000_000_000.0f64
        // divid slower than multiply
        self.duration as f64 * 0.000_000_001f64
    }

    /// Measure this function
    ///
    /// Use `Measure::this()` when you have a function that you want to measure.  `this()` will
    /// start a new `Measure`, call your function, stop the measure, then return the `Measure`
    /// object along with your function's return value.
    ///
    /// If your function takes more than one parameter, you will need to wrap your function in a
    /// closure, and wrap the arguments in a tuple.  The same thing applies to methods.  See the
    /// tests for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// // Call a function with a single argument
    /// # use crate::measure::Measure;
    /// # fn my_function(fizz: i32) -> i32 { fizz }
    /// let (result, measure) = Measure::this(my_function, 42, "my_func");
    /// # assert_eq!(result, 42);
    /// ```
    ///
    /// ```
    /// // Call a function with multiple arguments
    /// # use crate::measure::Measure;
    /// let (result, measure) = Measure::this(|(arg1, arg2)| std::cmp::min(arg1, arg2), (42, 123), "minimum");
    /// # assert_eq!(result, 42);
    /// ```
    ///
    /// ```
    /// // Call a method
    /// # use crate::measure::Measure;
    /// # struct Foo { x: i32 }
    /// # impl Foo { fn bar(&self, arg: i32) -> i32 { self.x + arg } }
    /// # let baz = 8;
    /// let foo = Foo { x: 42 };
    /// let (result, measure) = Measure::this(|(this, args)| Foo::bar(&this, args), (&foo, baz), "Foo::bar");
    /// # assert_eq!(result, 50);
    /// ```
    pub fn this<T, R, F: FnOnce(T) -> R>(func: F, args: T, name: &'static str) -> (R, Self) {
        let mut measure = Self::start(name);
        let result = func(args);
        measure.stop();
        (result, measure)
    }
}

impl fmt::Display for Measure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.duration == 0 {
            write!(f, "{} running", self.name)
        } else if self.as_us() < 1 {
            write!(f, "{} took {}ns", self.name, self.duration)
        } else if self.as_ms() < 1 {
            write!(f, "{} took {}us", self.name, self.as_us())
        } else if self.as_s() < 1. {
            write!(f, "{} took {}ms", self.name, self.as_ms())
        } else {
            write!(f, "{} took {:.1}s", self.name, self.as_s())
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::{thread::sleep, time::Duration},
    };

    #[test]
    fn test_measure() {
        let mut measure = Measure::start("test");
        sleep(Duration::from_secs(1));
        measure.stop();
        assert!(measure.as_s() >= 0.99f64 && measure.as_s() <= 1.01f64);
        assert!(measure.as_ms() >= 990 && measure.as_ms() <= 1_010);
        assert!(measure.as_us() >= 999_000 && measure.as_us() <= 1_010_000);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_measure_tsc() {
        use super::{MeasureTsc, x86::tsc_status};
        if super::x86::check_rdtscp() {
            println!("tsc status: {}", tsc_status());
        } else {
            println!("NO rdtscp SUPPORT");
            return
        }
        let mut measure = MeasureTsc::start("test");
        sleep(Duration::from_secs(1));
        measure.stop();
        assert!(measure.as_ms() >= 990 && measure.as_ms() <= 1_010);
        assert!(measure.as_us() >= 999_000 && measure.as_us() <= 1_010_000);
    }

    #[test]
    fn test_measure_display() {
        let measure = Measure {
            name: "test_ns",
            start: Instant::now(),
            duration: 1,
        };
        assert_eq!(format!("{}", measure), "test_ns took 1ns");

        let measure = Measure {
            name: "test_us",
            start: Instant::now(),
            duration: 1000,
        };
        assert_eq!(format!("{}", measure), "test_us took 1us");

        let measure = Measure {
            name: "test_ms",
            start: Instant::now(),
            duration: 1000 * 1000,
        };
        assert_eq!(format!("{}", measure), "test_ms took 1ms");

        let measure = Measure {
            name: "test_s",
            start: Instant::now(),
            duration: 1000 * 1000 * 1000,
        };
        assert_eq!(format!("{}", measure), "test_s took 1.0s");

        let measure = Measure::start("test_not_stopped");
        assert_eq!(format!("{}", measure), "test_not_stopped running");
    }

    fn my_multiply(x: i32, y: i32) -> i32 {
        x * y
    }

    fn my_multiply_tuple(args: (i32, i32)) -> i32 {
        let (x, y) = args;
        my_multiply(x, y)
    }

    fn square(x: i32) -> i32 {
        my_multiply(x, x)
    }

    struct SomeStruct {
        x: i32,
    }
    impl SomeStruct {
        fn add_to(&self, x: i32) -> i32 {
            x + self.x
        }
    }

    #[test]
    fn test_measure_this() {
        // Ensure that the measurement side actually works
        {
            let (_result, measure) = Measure::this(|s| sleep(Duration::from_secs(s)), 1, "test");
            assert!(measure.as_s() >= 0.99f64 && measure.as_s() <= 1.01f64);
            assert!(measure.as_ms() >= 990 && measure.as_ms() <= 1_010);
            assert!(measure.as_us() >= 999_000 && measure.as_us() <= 1_010_000);
        }

        // Ensure that this() can be called with a simple closure
        {
            let expected = 1;
            let (actual, _measure) = Measure::this(|x| x, expected, "test");
            assert_eq!(actual, expected);
        }

        // Ensure that this() can be called with a tuple
        {
            let (result, _measure) = Measure::this(|(x, y)| x + y, (1, 2), "test");
            assert_eq!(result, 1 + 2);
        }

        // Ensure that this() can be called with a normal function
        {
            let (result, _measure) = Measure::this(|(x, y)| my_multiply(x, y), (3, 4), "test");
            assert_eq!(result, 3 * 4);
        }

        // Ensure that this() can be called with a normal function with one argument
        {
            let (result, _measure) = Measure::this(square, 5, "test");
            assert_eq!(result, 5 * 5)
        }

        // Ensure that this() can be called with a normal function
        {
            let (result, _measure) = Measure::this(my_multiply_tuple, (3, 4), "test");
            assert_eq!(result, 3 * 4);
        }

        // Ensure that this() can be called with a method (and self)
        {
            let some_struct = SomeStruct { x: 42 };
            let (result, _measure) = Measure::this(
                |(obj, x)| SomeStruct::add_to(&obj, x),
                (some_struct, 4),
                "test",
            );
            assert_eq!(result, 42 + 4);
        }

        // Ensure that this() can be called with a method (and &self)
        {
            let some_struct = SomeStruct { x: 42 };
            let (result, _measure) = Measure::this(
                |(obj, x)| SomeStruct::add_to(obj, x),
                (&some_struct, 4),
                "test",
            );
            assert_eq!(result, 42 + 4);
            assert_eq!(some_struct.add_to(6), 42 + 6);
        }
    }
}
