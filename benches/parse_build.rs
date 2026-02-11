//! Pure Rust benchmark — no napi/JS overhead.
//!
//! Run: cargo bench --no-default-features --bench parse_build

use std::fs;
use std::time::Instant;

const WARMUP: usize = 10;
const ITERATIONS: usize = 500;

fn median(times: &mut Vec<f64>) -> f64 {
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}

fn bench<F: FnMut()>(mut f: F) -> f64 {
    for _ in 0..WARMUP {
        f();
    }
    let mut times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        f();
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    median(&mut times)
}

fn main() {
    let fixtures = [
        ("swift-protobuf.pbxproj", "257 KB"),
        ("Cocoa-Application.pbxproj", "166 KB"),
        ("AFNetworking.pbxproj", "99 KB"),
        ("watch.pbxproj", "48 KB"),
        ("project.pbxproj", "19 KB"),
    ];

    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/__test__/fixtures");

    println!("================================================================");
    println!(" Pure Rust Benchmark (no napi/JS overhead)");
    println!("================================================================");
    println!("Warmup: {}, Iterations: {}", WARMUP, ITERATIONS);
    println!();

    for (fixture, size) in &fixtures {
        let path = format!("{}/{}", fixtures_dir, fixture);
        let content = fs::read_to_string(&path).unwrap();
        let mb = content.len() as f64 / (1024.0 * 1024.0);

        let lex_med = bench(|| {
            let mut lexer = xcode::parser::lexer::Lexer::new(&content);
            let _ = lexer.tokenize_all().unwrap();
        });

        let parse_med = bench(|| {
            let _ = xcode::parser::parse(&content).unwrap();
        });

        let parsed = xcode::parser::parse(&content).unwrap();
        let build_med = bench(|| {
            let _ = xcode::writer::serializer::build(&parsed);
        });

        let rt_med = bench(|| {
            let p = xcode::parser::parse(&content).unwrap();
            let _ = xcode::writer::serializer::build(&p);
        });

        // Also bench JSON deser path (serde)
        let json = serde_json::to_string(&parsed).unwrap();
        let json_deser_med = bench(|| {
            let _: xcode::types::plist::PlistValue = serde_json::from_str(&json).unwrap();
        });

        let json_deser_build_med = bench(|| {
            let p: xcode::types::plist::PlistValue = serde_json::from_str(&json).unwrap();
            let _ = xcode::writer::serializer::build(&p);
        });

        println!("─ {} ({}) ─", fixture, size);
        println!(
            "  Lex:        {:>7.3} ms  ({:.0} MB/s)",
            lex_med,
            mb / (lex_med / 1000.0)
        );
        println!(
            "  Parse:      {:>7.3} ms  ({:.0} MB/s)",
            parse_med,
            mb / (parse_med / 1000.0)
        );
        println!(
            "  Build:      {:>7.3} ms  ({:.0} MB/s)",
            build_med,
            mb / (build_med / 1000.0)
        );
        println!("  Round-trip: {:>7.3} ms  ({:.0} MB/s)", rt_med, mb / (rt_med / 1000.0));
        println!(
            "  JSON deser: {:>7.3} ms  (serde_json::from_str → PlistValue)",
            json_deser_med,
        );
        println!("  JSON→build: {:>7.3} ms  (serde deser + build)", json_deser_build_med,);
        println!();
    }
}
