use rand::Rng;
use sha2::{Digest, Sha256};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Instant;

fn banner(title: &str) {
    println!("\n========================================================");
    println!("🔧 {}", title);
    println!("========================================================\n");
}

//
//  TESTE 1 -------------------------------------------
//  Força bruta em n³ + 17n + 12345
//
fn brute_force_chunk(start: u64, end: u64, stop: Arc<AtomicBool>) -> Option<u64> {
    for n in start..=end {
        if stop.load(Ordering::Relaxed) {
            return None;
        }

        let v = n.wrapping_mul(n).wrapping_mul(n) + 17 * n + 12_345;

        if v % 97_000_000 == 0 {
            stop.store(true, Ordering::Relaxed);
            return Some(n);
        }
    }
    None
}

//
//  TESTE 2 -------------------------------------------
//  Hashing SHA-256 em loop
//
fn sha256_burn(iterations: u64) {
    let mut hasher = Sha256::new();
    let mut data = vec![0u8; 4096];

    for _ in 0..iterations {
        rand::thread_rng().fill(&mut data[..]);
        hasher.update(&data);
        let _ = hasher.finalize_reset();
    }
}

//
//  TESTE 3 -------------------------------------------
//  Multiplicação de matrizes grande
//
fn matmul(n: usize) {
    let mut a = vec![vec![0.0; n]; n];
    let mut b = vec![vec![0.0; n]; n];
    let mut c = vec![vec![0.0; n]; n];

    let mut rng = rand::thread_rng();

    for i in 0..n {
        for j in 0..n {
            a[i][j] = rng.gen_range(0.0..1.0);
            b[i][j] = rng.gen_range(0.0..1.0);
        }
    }

    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a[i][k] * b[k][j];
            }
            c[i][j] = sum;
        }
    }
}

//
//  EXECUÇÃO DOS TESTES -------------------------------
//
fn main() {
    println!("🚀 BENCHMARK DE CPU — Rust Edition");
    println!("Detectando núcleos…");

    let threads = num_cpus::get();
    println!("💡 CPU reports {} logical cores\n", threads);

    // --------------------------
    // TESTE 1: Força Bruta
    // --------------------------
    banner("TESTE 1 — Força Bruta Inteira");

    let limit = 250_000_000u64;

    // SINGLE THREAD
    let start = Instant::now();
    let mut single_result = None;

    for n in 1..limit {
        let v = n.wrapping_mul(n).wrapping_mul(n) + 17 * n + 12_345;
        if v % 97_000_000 == 0 {
            single_result = Some(n);
            break;
        }
    }

    let single_time = start.elapsed();

    println!("🧵 Single-thread: {:?} — {:?}", single_result, single_time);

    // MULTI THREAD
    let chunk = limit / threads as u64;
    let stop_flag = Arc::new(AtomicBool::new(false));

    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..threads {
        let start_n = i as u64 * chunk + 1;
        let end_n = if i == threads - 1 {
            limit
        } else {
            (i as u64 + 1) * chunk
        };
        let flag = Arc::clone(&stop_flag);

        handles.push(thread::spawn(move || {
            brute_force_chunk(start_n, end_n, flag)
        }));
    }

    let mut multi_result = None;
    for h in handles {
        if let Ok(Some(v)) = h.join() {
            multi_result = Some(v);
        }
    }

    let multi_time = start.elapsed();

    println!("🧵 Multi-thread:  {:?} — {:?}", multi_result, multi_time);

    // --------------------------
    // TESTE 2: SHA-256
    // --------------------------
    banner("TESTE 2 — SHA-256 Hashing");

    let iters = 5_000u64;

    // Single
    let start = Instant::now();
    sha256_burn(iters);
    let t1 = start.elapsed();

    // Multi
    let start = Instant::now();
    let mut threads_sha = vec![];
    for _ in 0..threads {
        threads_sha.push(thread::spawn(move || sha256_burn(iters / 4)));
    }
    for h in threads_sha {
        h.join().unwrap();
    }
    let t2 = start.elapsed();

    println!(
        "🧵 SHA256 Single-thread: {:?}\n🧵 SHA256 Multi-thread:  {:?}",
        t1, t2
    );

    // --------------------------
    // TESTE 3: Matriz
    // --------------------------
    banner("TESTE 3 — Multiplicação de Matrizes 512×512");

    let size = 512;

    // Single
    let start = Instant::now();
    matmul(size);
    let t1 = start.elapsed();

    // Multi (divide linhas)
    let start = Instant::now();
    let mut th = vec![];

    for _ in 0..threads {
        th.push(thread::spawn(move || matmul(size / 2)));
    }

    for h in th {
        h.join().unwrap();
    }

    let t2 = start.elapsed();

    println!(
        "🧵 MatMul Single-thread: {:?}\n🧵 MatMul Multi-thread:  {:?}",
        t1, t2
    );

    println!("\n🏁 FIM DO BENCHMARK");
}
