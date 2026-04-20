// Hand-written Rust equivalent of bench_classes.liva
use std::time::Instant;

#[derive(Clone)]
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64),
}

fn shape_area(s: &Shape) -> f64 {
    match s {
        Shape::Circle(r) => std::f64::consts::PI * r * r,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(b, h) => 0.5 * b * h,
    }
}

fn shape_perimeter(s: &Shape) -> f64 {
    match s {
        Shape::Circle(r) => 2.0 * std::f64::consts::PI * r,
        Shape::Rectangle(w, h) => 2.0 * (w + h),
        Shape::Triangle(b, h) => b + h + (b * b + h * h).sqrt(),
    }
}

#[derive(Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self { Vec2 { x, y } }
    fn add(&self, other: &Vec2) -> Vec2 { Vec2::new(self.x + other.x, self.y + other.y) }
    fn scale(&self, factor: f64) -> Vec2 { Vec2::new(self.x * factor, self.y * factor) }
    fn magnitude(&self) -> f64 { (self.x * self.x + self.y * self.y).sqrt() }
    #[allow(dead_code)]
    fn dot(&self, other: &Vec2) -> f64 { self.x * other.x + self.y * other.y }
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    mass: f64,
}

impl Particle {
    fn new(x: f64, y: f64, vx: f64, vy: f64, mass: f64) -> Self {
        Particle { pos: Vec2::new(x, y), vel: Vec2::new(vx, vy), mass }
    }
    fn step(&mut self, dt: f64) {
        let dx = self.vel.scale(dt);
        self.pos = self.pos.add(&dx);
    }
    fn kinetic_energy(&self) -> f64 {
        let v = self.vel.magnitude();
        0.5 * self.mass * v * v
    }
}

fn main() {
    let iterations = 1000;

    // Benchmark 1: Shape area/perimeter computation
    let mut shapes = Vec::new();
    for i in 0..1000 {
        let fi = i as f64;
        shapes.push(Shape::Circle(fi * 0.1 + 1.0));
        shapes.push(Shape::Rectangle(fi * 0.2 + 1.0, fi * 0.3 + 1.0));
        shapes.push(Shape::Triangle(fi * 0.15 + 1.0, fi * 0.25 + 1.0));
    }

    let t1 = Instant::now();
    for _ in 0..iterations {
        let mut total_area = 0.0_f64;
        let mut total_perim = 0.0_f64;
        for shape in &shapes {
            total_area += shape_area(shape);
            total_perim += shape_perimeter(shape);
        }
    }
    println!("Shape compute: {}ms ({} x 3000 shapes)", t1.elapsed().as_millis(), iterations);

    // Benchmark 2: Vec2 operations
    let t2 = Instant::now();
    for _ in 0..iterations {
        let mut v1 = Vec2::new(1.0, 2.0);
        for _ in 0..10000 {
            let v2 = Vec2::new(0.1, 0.2);
            v1 = v1.add(&v2);
            v1 = v1.scale(0.999);
            let _ = v1.magnitude();
        }
    }
    println!("Vec2 ops: {}ms ({} x 10000 ops)", t2.elapsed().as_millis(), iterations);

    // Benchmark 3: Particle simulation
    let mut particles = Vec::new();
    for i in 0..100 {
        let fi = i as f64;
        particles.push(Particle::new(fi, fi * 0.5, 1.0, 0.5, fi + 1.0));
    }

    let t3 = Instant::now();
    for _ in 0..iterations {
        for p in particles.iter_mut() {
            for _ in 0..100 {
                p.step(0.01);
                let _ = p.kinetic_energy();
            }
        }
    }
    println!("Particle sim: {}ms ({} x 100 particles x 100 steps)", t3.elapsed().as_millis(), iterations);
}
