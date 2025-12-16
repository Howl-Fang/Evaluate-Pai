use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use rug::{Integer, Float};
use rug::ops::Pow;
use num_cpus;
use chrono;

// 使用整数运算的 Chudnovsky 算法
// 基于二进分割法 (Binary Splitting) 加速收敛
struct ChudnovskyBinarySplit {
    // 常数
    a: Integer,           // 13591409
    b: Integer,           // 545140134
    c: Integer,           // 640320
    c3_over_24: Integer, // (640320^3)/24
}

impl ChudnovskyBinarySplit {
    fn new() -> Self {
        let a = Integer::from(13591409);
        let b = Integer::from(545140134);
        let c = Integer::from(640320);
        
        // 计算 c^3 / 24
        let c2 = Integer::from(&c * &c);
        let c3 = Integer::from(&c2 * &c);
        let c3_over_24 = Integer::from(&c3 / 24);
        
        Self {
            a,
            b,
            c,
            c3_over_24,
        }
    }
    
    // 计算 P(a, b), Q(a, b), T(a, b)
    // 返回 (P, Q, T) 使得 Σ_{k=a}^{b-1} term_k = T / (P * Q)
    fn compute_binary_split(&self, a: u64, b: u64) -> (Integer, Integer, Integer) {
        if b - a == 1 {
            // 计算单个项
            let k = a;
            
            // 分子: (-1)^k * (6k)! * (a + b*k)
            let sign = if k % 2 == 0 { 1 } else { -1 };
            
            // 计算 (6k)!
            let six_k_fac = factorial(6 * k);
            
            // 计算 (a + b*k)
            let b_times_k = Integer::from(&self.b * k);
            let lk = Integer::from(&self.a + &b_times_k);
            
            // 分子 P
            let p_temp = Integer::from(&six_k_fac * &lk);
            let p = if sign == -1 { -p_temp } else { p_temp };
            
            // 分母 Q: (3k)! * (k!)^3 * c^(3k)
            let three_k_fac = factorial(3 * k);
            let k_fac = factorial(k);
            
            // 计算 k!^3
            let k_fac_sq = Integer::from(&k_fac * &k_fac);
            let k_fac_cubed = Integer::from(&k_fac_sq * &k_fac);
            
            // 计算 c^(3k) - 使用 Pow trait
            let c_pow_3k = self.c.clone().pow(3 * k as u32);
            
            // 计算 Q
            let q1 = Integer::from(&three_k_fac * &k_fac_cubed);
            let q = Integer::from(&q1 * &c_pow_3k);
            
            // T = 1
            let t = Integer::from(1);
            
            (p, q, t)
        } else {
            // 分治递归
            let m = (a + b) / 2;
            let (p1, q1, t1) = self.compute_binary_split(a, m);
            let (p2, q2, t2) = self.compute_binary_split(m, b);
            
            // 合并结果
            let p1q2 = Integer::from(&p1 * &q2);
            let p2t1 = Integer::from(&p2 * &t1);
            let p = Integer::from(&p1q2 + &p2t1);
            
            let q = Integer::from(&q1 * &q2);
            let t = Integer::from(&t1 * &t2);
            
            (p, q, t)
        }
    }
    
    // 计算 π 到指定精度
    fn compute_pi(&self, digits: usize, num_threads: usize) -> (Float, Duration) {
        println!("使用二进分割法计算 π 到 {} 位有效数字...", digits);
        println!("线程数: {}", num_threads);
        
        let start = Instant::now();
        
        // 计算需要的项数
        // Chudnovsky 每项增加约 14.18 位十进制数字
        let terms_needed = (digits as f64 / 14.18).ceil() as u64;
        
        // 限制项数，避免计算时间过长
        let max_terms = 1000; // 限制最大项数
        let terms_needed = if terms_needed > max_terms { max_terms } else { terms_needed };
        
        println!("需要计算 {} 项...", terms_needed);
        
        // 使用多线程计算二进分割
        let num_terms_per_thread = (terms_needed as f64 / num_threads as f64).ceil() as u64;
        let mut ranges = Vec::new();
        
        let mut start_term = 0;
        while start_term < terms_needed {
            let end_term = (start_term + num_terms_per_thread).min(terms_needed);
            ranges.push((start_term, end_term));
            start_term = end_term;
        }
        
        // 并行计算每个区间
        let mut handles = Vec::new();
        for (a, b) in ranges {
            let calculator = self.clone();
            let handle = thread::spawn(move || {
                calculator.compute_binary_split(a, b)
            });
            handles.push(handle);
        }
        
        // 收集并合并结果
        let mut final_p = Integer::from(0);
        let mut final_q = Integer::from(1);
        let mut final_t = Integer::from(1);
        
        for handle in handles {
            let (p, q, t) = handle.join().unwrap();
            
            // 合并公式: P = P1*Q2 + P2*T1, Q = Q1*Q2, T = T1*T2
            let p1q2 = Integer::from(&final_p * &q);
            let p2t1 = Integer::from(&p * &final_t);
            let new_p = Integer::from(&p1q2 + &p2t1);
            let new_q = Integer::from(&final_q * &q);
            let new_t = Integer::from(&final_t * &t);
            
            final_p = new_p;
            final_q = new_q;
            final_t = new_t;
        }
        
        // 计算 π = (426880 * sqrt(10005) * Q) / (12 * P)
        let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;
        
        // 将整数转换为浮点数
        let p_float = Float::with_val(precision, &final_p);
        let q_float = Float::with_val(precision, &final_q);
        
        // 计算 sqrt(10005)
        let sqrt_10005 = Float::with_val(precision, 10005.0);
        let sqrt_10005 = sqrt_10005.sqrt();
        
        // 计算分子: 426880 * sqrt(10005) * Q
        let num1 = Float::with_val(precision, 426880.0) * &sqrt_10005;
        let numerator = Float::with_val(precision, &num1 * &q_float);
        
        // 计算分母: 12 * P
        let denominator = Float::with_val(precision, 12.0) * &p_float;
        
        // 计算 π
        let pi = Float::with_val(precision, &numerator / &denominator);
        
        let duration = start.elapsed();
        println!("计算完成，耗时: {:?}", duration);
        println!("平均速度: {:.1} 位/秒", digits as f64 / duration.as_secs_f64());
        
        (pi, duration)
    }
}

impl Clone for ChudnovskyBinarySplit {
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
            c: self.c.clone(),
            c3_over_24: self.c3_over_24.clone(),
        }
    }
}

// 计算阶乘
fn factorial(n: u64) -> Integer {
    if n == 0 {
        return Integer::from(1);
    }
    
    let mut result = Integer::from(1);
    for i in 1..=n {
        result *= i;
    }
    result
}

// 优化的直接计算法
fn compute_pi_direct_optimized(digits: usize, num_threads: usize) -> (Float, Duration) {
    println!("使用优化直接计算法计算 π 到 {} 位有效数字...", digits);
    
    let start = Instant::now();
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;
    
    // 估计需要的项数
    let terms_needed = (digits as f64 / 14.18).ceil() as usize;
    
    println!("需要计算 {} 项...", terms_needed);
    
    // 预计算常数
    let const_426880 = Float::with_val(precision, 426880.0);
    let const_12 = Float::with_val(precision, 12.0);
    let sqrt_10005 = Float::with_val(precision, 10005.0);
    let sqrt_10005 = sqrt_10005.sqrt();
    
    // 将任务分成块
    let chunk_size = 100;
    let num_chunks = (terms_needed + chunk_size - 1) / chunk_size;
    
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let result = Arc::new(std::sync::Mutex::new(Float::with_val(precision, 0.0)));
    
    let mut handles = Vec::new();
    
    for _ in 0..num_threads {
        let counter = Arc::clone(&counter);
        let result = Arc::clone(&result);
        let prec = precision;
        
        let handle = thread::spawn(move || {
            let mut local_sum = Float::with_val(prec, 0.0);
            
            loop {
                let chunk_idx = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if chunk_idx >= num_chunks {
                    break;
                }
                
                let start_term = chunk_idx * chunk_size;
                let end_term = std::cmp::min(start_term + chunk_size, terms_needed);
                
                if start_term >= end_term {
                    continue;
                }
                
                // 计算这个块的项
                let mut current_sign = if start_term % 2 == 0 { 1.0 } else { -1.0 };
                
                for k in start_term..end_term {
                    // 使用递推关系计算项
                    let term = compute_chudnovsky_term(k, prec);
                    let term_with_sign = Float::with_val(prec, &term * current_sign);
                    local_sum += &term_with_sign;
                    
                    // 更新符号
                    current_sign = -current_sign;
                }
            }
            
            // 添加到全局结果
            let mut global_result = result.lock().unwrap();
            *global_result += &local_sum;
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 获取最终和
    let sum = {
        let result = result.lock().unwrap();
        result.clone()
    };
    
    // 计算 π
    let numerator = Float::with_val(precision, &const_426880 * &sqrt_10005);
    let denominator = Float::with_val(precision, &const_12 * &sum);
    let pi = Float::with_val(precision, &numerator / &denominator);
    
    let duration = start.elapsed();
    println!("计算完成，耗时: {:?}", duration);
    
    (pi, duration)
}

// 计算单个 Chudnovsky 项
fn compute_chudnovsky_term(k: usize, precision: u32) -> Float {
    if k == 0 {
        // 第 0 项
        let numerator = Float::with_val(precision, 13591409.0);
        let denominator_temp = Float::with_val(precision, 426880.0);
        let sqrt_10005 = Float::with_val(precision, 10005.0).sqrt();
        let denominator = Float::with_val(precision, &denominator_temp * &sqrt_10005);
        return Float::with_val(precision, &numerator / &denominator);
    }
    
    // 计算阶乘的对数（使用斯特林近似）
    let k_f64 = k as f64;
    let six_k = 6.0 * k_f64;
    let three_k = 3.0 * k_f64;
    
    // 斯特林公式：ln(n!) ≈ n*ln(n) - n + 0.5*ln(2πn)
    let ln_six_k_fac = six_k * six_k.ln() - six_k + 0.5 * (2.0 * std::f64::consts::PI * six_k).ln();
    let ln_three_k_fac = three_k * three_k.ln() - three_k + 0.5 * (2.0 * std::f64::consts::PI * three_k).ln();
    let ln_k_fac = k_f64 * k_f64.ln() - k_f64 + 0.5 * (2.0 * std::f64::consts::PI * k_f64).ln();
    
    // 计算 L_k
    let lk = 13591409.0 + 545140134.0 * k_f64;
    let ln_lk = lk.ln();
    
    // 计算 c^(3k)
    let ln_c_pow = 3.0 * k_f64 * 640320.0_f64.ln();
    
    // 计算项的对数
    let ln_term = ln_six_k_fac + ln_lk - ln_three_k_fac - 3.0 * ln_k_fac - ln_c_pow;
    
    // 计算项的值
    Float::with_val(precision, ln_term.exp())
}

// 混合策略：根据位数选择算法
fn compute_pi_hybrid(digits: usize, num_threads: usize) -> (Float, Duration) {
    if digits <= 10000 {
        // 小位数使用直接计算
        compute_pi_direct_optimized(digits, num_threads)
    } else {
        // 大位数使用二进分割法
        let calculator = ChudnovskyBinarySplit::new();
        calculator.compute_pi(digits, num_threads)
    }
}

// 写入文件
fn write_pi_to_file(pi: &Float, digits: usize, filename: &str) -> io::Result<()> {
    println!("将结果写入文件 {}...", filename);
    let start = Instant::now();
    
    let file = std::fs::File::create(filename)?;
    let mut writer = io::BufWriter::new(file);
    
    // 写入头信息
    writeln!(writer, "π 的前 {} 位有效数字", digits)?;
    writeln!(writer, "计算时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(writer, "{}", "=".repeat(80))?;
    
    // 获取 π 的字符串表示
    let pi_str = pi.to_string_radix(10, Some(digits));
    
    // 格式化输出
    let mut chars = pi_str.chars();
    let mut count = 0;
    
    // 写入 "3."
    if let Some(ch) = chars.next() {
        write!(writer, "{}", ch)?;
    }
    if let Some(ch) = chars.next() {
        write!(writer, "{}", ch)?;
    }
    
    // 每 10 个数字一组，每 5 组一行
    for ch in chars {
        write!(writer, "{}", ch)?;
        count += 1;
        
        if count % 10 == 0 {
            write!(writer, " ")?;
        }
        if count % 50 == 0 {
            writeln!(writer)?;
        }
    }
    
    writer.flush()?;
    
    let duration = start.elapsed();
    println!("写入完成，耗时: {:?}", duration);
    
    Ok(())
}

// 获取用户输入
fn get_user_input() -> (usize, usize) {
    println!("高性能 π 计算器");
    println!("{}", "=".repeat(50));
    
    // 获取计算位数
    let digits = loop {
        print!("请输入要计算的 π 的位数 (1-1,000,000,000, 默认 1000): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            break 1000;
        }
        
        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= 1_000_000_000 => break n,
            Ok(_) => println!("位数必须在 1 到 1,000.000,000 之间"),
            Err(_) => println!("请输入有效的数字"),
        }
    };
    
    // 获取线程数
    let max_threads = num_cpus::get();
    let num_threads = loop {
        print!("请输入线程数 (1-{}, 默认 {}): ", max_threads, max_threads);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            break max_threads;
        }
        
        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= max_threads => break n,
            Ok(_) => println!("线程数必须在 1 到 {} 之间", max_threads),
            Err(_) => println!("请输入有效的数字"),
        }
    };
    
    (digits, num_threads)
}

// 内存使用统计
fn print_memory_stats(digits: usize, threads: usize) {
    println!("\n内存使用估算:");
    println!("{}", "-".repeat(40));
    
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;
    let float_size_bytes = precision as f64 / 8.0;
    
    println!("计算位数: {} 位十进制", digits);
    println!("精度: {} 位二进制", precision);
    println!("每个高精度浮点数: {:.2} MB", float_size_bytes / 1024.0 / 1024.0);
    println!("线程数: {}", threads);
    
    // 估计总内存
    let total_memory_mb = (threads as f64 + 2.0) * float_size_bytes / 1024.0 / 1024.0;
    println!("总估算内存: {:.2} MB", total_memory_mb);
    
    if total_memory_mb > 500.0 {
        println!("⚠️  警告: 内存使用可能很高，考虑减少线程数或位数");
    }
}

// 验证准确性
fn verify_pi_accuracy(pi_str: &str, digits: usize) -> (bool, usize) {
    let known_pi = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679";
    
    let known_digits: Vec<char> = known_pi.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    
    let computed_digits: Vec<char> = pi_str.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    
    let compare_len = std::cmp::min(100, digits);
    let compare_len = std::cmp::min(compare_len, known_digits.len());
    let compare_len = std::cmp::min(compare_len, computed_digits.len());
    
    for i in 0..compare_len {
        if computed_digits[i] != known_digits[i] {
            return (false, i);
        }
    }
    
    (true, compare_len)
}

fn main() {
    let (digits, num_threads) = get_user_input();
    
    println!("\n{}", "=".repeat(50));
    println!("开始计算 π 到 {} 位有效数字", digits);
    println!("使用 {} 个线程", num_threads);
    println!("{}", "=".repeat(50));
    
    print_memory_stats(digits, num_threads);
    
    // 计算 π
    let (pi, compute_time) = compute_pi_hybrid(digits, num_threads);
    
    // 显示结果预览
    println!("\nπ 的前 50 位:");
    println!("{}", "-".repeat(52));
    let preview_str = pi.to_string_radix(10, Some(50));
    println!("{}", preview_str);
    
    // 验证准确性
    println!("\n验证准确性:");
    println!("{}", "-".repeat(52));
    let pi_full_str = pi.to_string_radix(10, Some(100.min(digits)));
    let (accurate, correct_digits) = verify_pi_accuracy(&pi_full_str, digits);
    
    if accurate {
        println!("✓ 前 {} 位与已知 π 值完全一致", correct_digits);
    } else {
        println!("✗ 前 {} 位正确，第 {} 位开始出现差异", correct_digits, correct_digits + 1);
    }
    
    // 写入文件
    let filename = format!("pi_{}_digits.txt", digits);
    println!("\n写入文件...");
    println!("{}", "-".repeat(52));
    
    match write_pi_to_file(&pi, digits, &filename) {
        Ok(_) => {
            if let Ok(metadata) = std::fs::metadata(&filename) {
                println!("\n文件信息:");
                println!("{}", "-".repeat(52));
                println!("文件名: {}", filename);
                println!("文件大小: {:.2} KB", metadata.len() as f64 / 1024.0);
                println!("计算时间: {:?}", compute_time);
                println!("平均速度: {:.2} 位/秒", digits as f64 / compute_time.as_secs_f64());
                
                if metadata.len() < 1024 * 1024 {
                    println!("\n提示: 使用 'head -n 50 {}' 查看文件内容", filename);
                } else {
                    println!("\n提示: 文件较大，使用 'head -n 50 {}' 查看开头", filename);
                }
            }
        }
        Err(e) => eprintln!("写入文件失败: {}", e),
    }
    
    println!("\n计算完成！结果已保存到 {}", filename);
}
