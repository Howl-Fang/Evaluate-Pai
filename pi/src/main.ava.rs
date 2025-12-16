use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;
use rug::{Float, ops::Pow, Assign};
use rug::float::Round;

// 计算 π 的 BBP 公式的项
fn bbp_term(k: usize, precision: u32) -> Float {
    let prec = precision + 10; // 额外精度防止舍入误差
    
    let kf = Float::with_val(prec, k);
    let eight_k = Float::with_val(prec, 8 * k);
    
    // 计算 4/(8k+1) - 2/(8k+4) - 1/(8k+5) - 1/(8k+6)
    let term1 = Float::with_val(prec, 4) / (eight_k.clone() + 1);
    let term2 = Float::with_val(prec, 2) / (eight_k.clone() + 4);
    let term3 = Float::with_val(prec, 1) / (eight_k.clone() + 5);
    let term4 = Float::with_val(prec, 1) / (eight_k + 6);
    
    // 乘以 1/16^k
    let sixteen_pow_k = Float::with_val(prec, 16).pow(-(k as i32));
    
    (term1 - term2 - term3 - term4) * sixteen_pow_k
}

// 并行计算 π
fn compute_pi_parallel(digits: usize, num_threads: usize) -> Float {
    println!("使用 {} 个线程计算 π 到 {} 位有效数字...", num_threads, digits);
    
    let start = Instant::now();
    
    // 计算所需精度（二进制位）
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;
    
    // 计算需要多少项才能达到所需精度
    // BBP 公式每项贡献约 4 位二进制位
    let terms_needed = (precision as usize) / 4 + 10;
    
    println!("需要计算 {} 项...", terms_needed);
    
    // 用于分发任务的原子计数器
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    // 创建线程
    for _ in 0..num_threads {
        let counter = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            let mut thread_sum = Float::with_val(precision, 0);
            
            loop {
                // 获取下一个要计算的任务
                let k = counter.fetch_add(1, Ordering::SeqCst);
                if k >= terms_needed {
                    break;
                }
                
                let term = bbp_term(k, precision);
                thread_sum += term;
            }
            
            thread_sum
        });
        
        handles.push(handle);
    }
    
    // 收集结果
    let mut pi = Float::with_val(precision, 0);
    for handle in handles {
        let thread_result = handle.join().unwrap();
        pi += thread_result;
    }
    
    let duration = start.elapsed();
    println!("计算完成，耗时: {:?}", duration);
    
    // 设置精度并返回结果
    pi.set_prec(precision);
    pi
}

fn main() {
    println!("π 计算器 (使用并行 BBP 算法)");
    println!("==============================");
    
    // 获取用户输入
    let digits = 1000; // 默认计算 1000 位
    println!("将计算 π 到 {} 位有效数字", digits);
    
    // 使用可用 CPU 核心数
    let num_threads = num_cpus::get();
    println!("检测到 {} 个 CPU 核心", num_threads);
    
    // 计算 π
    let pi = compute_pi_parallel(digits, num_threads);
    
    // 输出结果
    println!("\nπ 的前 {} 位:", digits.min(50));
    println!("==========================================");
    
    let pi_str = pi.to_string_radix(10, Some(digits));
    let pi_str = if pi_str.len() > 52 {
        format!("{}...", &pi_str[..52])
    } else {
        pi_str
    };
    
    println!("π = {}", pi_str);
    
    // 验证计算精度
    println!("\n验证:");
    println!("3.14159265358979323846264338327950288419716939937510...");
    println!("----------------------------------------------------------");
    
    // 如果需要，可以将完整结果保存到文件
    println!("\n如果需要保存完整结果到文件，请修改代码。");
}

// 可选的性能测试函数
#[allow(dead_code)]
fn benchmark() {
    println!("性能测试:");
    println!("=========");
    
    let test_digits = 1000;
    let test_threads = vec![1, 2, 4, 8];
    
    for &threads in &test_threads {
        println!("\n使用 {} 个线程:", threads);
        let start = Instant::now();
        let _pi = compute_pi_parallel(test_digits, threads);
        let duration = start.elapsed();
        println!("耗时: {:?}", duration);
    }
}
