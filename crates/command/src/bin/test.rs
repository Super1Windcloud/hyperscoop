#[allow(dead_code)]
fn main() {
  // 数值微分
  fn fx(x: f64) -> f64 {
    return x.powf(2.0) + 3.0 * x + 2.0;
  }
  fn numerical_derivative(f: fn(f64) -> f64, x: f64, h: f64) -> f64 {
    return (f(x + h) - f(x - h)) / (2.0 * h);
  }
  let x = 2.0;

  let derivative_at_x = numerical_derivative(fx, x, 1e-6);

  println!("The derivative of f(x) at x = {} is: {}", x, derivative_at_x);
}

