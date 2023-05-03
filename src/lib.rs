use std::{fmt::Display, ops::{Sub, Div}};
use num_dual::{DualNumFloat};

pub trait Derivable<T> where T: DualNumFloat {
    fn execute_derivative(&self) -> Self;
    fn zeroth_derivative(&self) -> T;
    fn first_derivative(&self) -> T;
    fn second_derivative(&self) -> T;
}

pub trait Coerceable<T> where T: DualNumFloat{
    fn coerce_to(&self) -> T;
    fn coerce_from(value: T) -> Self;
}

fn newton<'a, F, N, T>(f: F, guess: T, patience: i32, tolerance: T) -> Option<T>
where
    F: Fn(N) -> N + Send + Sync + 'a,
    N: Derivable<T> + Coerceable<T> + Display + Copy,
    T: DualNumFloat
{
    let mut current: T = guess;
    let mut count = 0;
    loop {
        count += 1;
        let x = N::coerce_from(current).execute_derivative();
        let z = f(x);
        let next = x.zeroth_derivative() - z.zeroth_derivative() / z.first_derivative();
        let diff = next - current;
        if diff.abs() < tolerance {
            println!("Found root at: {}", next);
            return Some(next);
        } else {
            if count > patience {
                println!("Failed to find root with initial guess of {}", guess);
                println!("Last iteration was: {}", current);
                println!("Try updating the initial guess or increasing the tolerance or patience");
                return None;
            }
            current = next;
        }
    }
}

fn find_bisections<'a, F, N, T>(f: F, lower: T, upper: T, resolution: i32) -> Vec<(T, T)>
where
    F: Fn(N) -> N + Sync + Send + Copy + 'a,
    N: Derivable<T> + Coerceable<T> + Display + Copy + Sub + Div,
    T: DualNumFloat
{
    let step = (upper - lower) / T::from(resolution).unwrap() + T::epsilon();
    // Add off-set to step to deal with roots at middle of lower and upper range
    let mut values: Vec<(T, T)> = Vec::new();

    for i in 0..resolution {
        let a = lower + step * T::from(i).unwrap();
        let b = lower + step * T::from(i+1).unwrap();
        let fa = f(N::coerce_from(a));
        let fb = f(N::coerce_from(b));
        let pos2neg = fa.zeroth_derivative() > T::zero() && fb.zeroth_derivative() < T::zero();
        let neg2pos = fa.zeroth_derivative() < T::zero() && fb.zeroth_derivative() > T::zero();
        if pos2neg || neg2pos {
            values.push((a, b));
        }
    };
    values
}

pub fn root_search<'a, F, N, T>(f: F, lower: T, upper: T, resolution: i32, patience: i32, tolerance: T) -> (Vec<T>, Vec<(T, T)>)
where
    F: Fn(N) -> N + Sync + Send + Copy + 'a,
    N: Derivable<T> + Coerceable<T> + Display + Copy + Sub + Div,
    T: DualNumFloat
{
    if lower > upper {
        panic!("Lower bound must be greater than upper bound")
    }
    if lower == upper {
        panic!("Bounds cannot be the same")
    }
    let bisections = find_bisections(f, lower, upper, resolution);
    let mut roots: Vec<T> = Vec::new();
    for bisection in &bisections {
        let res = T::from(100).unwrap();
        let step = (bisection.1 - bisection.0) / res;
        for i in 0..res.to_i32().unwrap() {
            let guess = bisection.0 + (T::from(i).unwrap() * step);
            let root = newton(f, guess, patience, tolerance);
            if root.is_none() {
                break;
            }
            let root = root.unwrap();
            if bisection.0 < root && root < bisection.1 {
                roots.push(root);
                break;
            } else if bisection.0 < root && root < bisection.1 {
                roots.push(root);
                break;
            }
        }

    }
    (roots, bisections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_dual::{Dual32, DualNumFloat, DualNum};

    impl Derivable<f32> for Dual32 {
        fn execute_derivative(&self) -> Self {
            return self.derive()
        }
        fn zeroth_derivative(&self) -> f32 {
            return self.re
        }
        fn first_derivative(&self) -> f32 {
            return self.eps[0]
        }
        fn second_derivative(&self) -> f32 {
            return self.eps[1]
        }
    }
    
    impl <T: DualNumFloat> Coerceable<T> for Dual32 {
        fn coerce_to(&self) -> T {
            return T::from(self.re).unwrap()
        }
        fn coerce_from(value: T) -> Self {
            return Dual32::from_re(value.to_f32().unwrap())
        }
    }

    #[test]
    fn find_sine_root_newton() {
        fn sine<D: DualNum<f32>>(x: D) -> D {
            x.sin()
        }
        let root = newton::<_,Dual32,f32>(&sine, 2.0, 1000, 0.0001);
        assert_eq!(std::f32::consts::PI, root.unwrap())
    }

    #[test]
    fn find_cosine_root_newton() {
        fn cosine<D: DualNum<f32>>(x: D) -> D {
            x.cos()
        }
        let root = newton::<_,Dual32,f32>(&cosine, 2.0, 1000, 0.0001);
        assert_eq!(std::f32::consts::PI / 2.0, root.unwrap())
    }

    #[test]
    fn find_sine_bisections() {
        fn sine<D: DualNum<f32>>(x: D) -> D {
            x.sin()
        }
        let bisections = find_bisections::<_,Dual32,f32>(&sine, -5.0, 5.0, 2000);
        for bisection in &bisections {
            println!("bisection: ({},{})", bisection.0, bisection.1)
        }
        assert_eq!(bisections.len(), 3)
    }

    #[test]
    fn find_cosine_bisections() {
        fn cosine<D: DualNum<f32>>(x: D) -> D {
            x.cos()
        }
        let bisections = find_bisections::<_,Dual32,f32>(&cosine, -5.0, 5.0, 2000);
        for bisection in &bisections {
            println!("bisection: ({},{})", bisection.0, bisection.1)
        }
        assert_eq!(bisections.len(), 4)
    }

    #[test]
    fn find_sine_roots() {
        fn sine<D: DualNum<f32>>(x: D) -> D {
            x.sin()
        }
        let roots = root_search::<_,Dual32,f32>(&sine, -5.0, 5.0, 2000, 1000, 0.0001);
        for root in &roots.0 {
            println!("root: {}", root);
        }
        assert_eq!(roots.0.len(), 3);
        assert!(roots.0.contains(&std::f32::consts::PI));
        assert!(roots.0.contains(&(-std::f32::consts::PI)));
        assert!(roots.0.contains(&0.0));
    }

    #[test]
    fn find_cosine_roots() {
        fn cosine<D: DualNum<f32>>(x: D) -> D {
            x.cos()
        }
        let roots = root_search::<_,Dual32,f32>(&cosine, -5.0, 5.0, 2000, 1000, 0.0001);
        for root in &roots.0 {
            println!("root: {}", root);
        }
        assert_eq!(roots.0.len(), 4);
        assert!(roots.0.contains(&std::f32::consts::FRAC_PI_2));
        assert!(roots.0.contains(&(-std::f32::consts::FRAC_PI_2)));
        assert!(roots.0.contains(&(std::f32::consts::FRAC_PI_2 * 3.0)));
        assert!(roots.0.contains(&(-std::f32::consts::FRAC_PI_2 * 3.0)));
    }


}