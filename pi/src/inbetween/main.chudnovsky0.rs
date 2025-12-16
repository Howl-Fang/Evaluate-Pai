use std::sync::Arc;
use std::thread;
use std::time::Instant;
use rug::{Float, Assign, ops::Pow};
use num_cpus;
use std::io::{self, Write};
use chrono;

// Chudnovsky 算法的递推实现
struct ChudnovskyIter {
    // 当前项的值
    current_term: Float,
    
    // 递推变量
    m_k: Float,      // M_k
    sign: i32,       // (-1)^k
    k: u64,          // 当前 k
    precision: u32,  // 计算精度
}

impl ChudnovskyIter {
    fn new(precision: u32) -> Self {
        let prec = precision;
        
        // 初始化 M_0 = 1
        let m_k = Float::with_val(prec, 1.0);
        
        // 计算第 0 项: L_0 / (426880 * sqrt(10005))
        // 其中 L_0 = 13591409
        let denominator = {
            let mut denom = Float::with_val(prec, 426880u32);
            let sqrt_10005 = Float::with_val(prec, 10005.0).sqrt();
            denom *= sqrt_10005;
            denom
        };
        
        let mut term_0 = Float::with_val(prec, 13591409u32);
        term_0 /= denominator;
        
        Self {
            current_term: term_0,
            m_k,
            sign: 1,
            k: 0,
            precision: prec,
        }
    }
    
    // 获取当前项
    fn current(&self) -> &Float {
        &self.current_term
    }
    
    // 前进到下一项
    fn next_term(&mut self) {
        self.k += 1;
        
        // 计算 L_k = 13591409 + 545140134*k
        let lk = 13591409.0 + 545140134.0 * (self.k as f64);
        
        // 计算递推因子: f_k = -(6k-5)(2k-1)(6k-1) / (k^3 * 640320^3/24)
        let k_f64 = self.k as f64;
        
        // 分子: (6k-5)(2k-1)(6k-1)
        let numerator = (6.0 * k_f64 - 5.0) * (2.0 * k_f64 - 1.0) * (6.0 * k_f64 - 1.0);
        
        // 分母: k^3 * 640320^3/24
        let k3 = k_f64 * k_f64 * k_f64;
        let c3_over_24 = 640320.0_f64.powi(3) / 24.0;
        let denominator = k3 * c3_over_24;
        
        // 递推因子
        let factor = -numerator / denominator;
        
        // 更新 M_k
        self.m_k *= factor;
        
        // 符号: (-1)^k
        self.sign = if self.k % 2 == 0 { 1 } else { -1 };
        
        // 计算当前项: (-1)^k * M_k * L_k
        self.current_term.assign(&self.m_k);
        self.current_term *= lk;
        self.current_term *= self.sign as f64;
    }
    
    // 计算从当前项开始的 n 项之和
    fn sum_next_n_terms(&mut self, n: usize) -> Float {
        let mut sum = Float::with_val(self.precision, 0.0);
        
        for _ in 0..n {
            sum += &self.current_term;
            self.next_term();
        }
        
        sum
    }
}

// 使用 Chudnovsky 算法计算 π
fn compute_pi_chudnovsky(digits: usize, num_threads: usize) -> (Float, f64) {
    println!("使用 Chudnovsky 算法计算 π 到 {} 位有效数字...", digits);
    println!("线程数: {}", num_threads);
    
    let start = Instant::now();
    
    // 计算所需精度
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;
    
    // 计算需要的项数
    // 每个项增加约 14.18 位十进制数字
    let terms_needed = (digits as f64 / 14.18).ceil() as usize + 2;
    
    println!("精度: {} 位二进制", precision);
    println!("需要计算 {} 项...", terms_needed);
    
    // 将项分成块
    let chunk_size = 100;  // 每块 100 项
    
    // 使用工作窃取模式
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
                // 获取下一个块
                let chunk_idx = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let start_term = chunk_idx * chunk_size;
                
                if start_term >= terms_needed {
                    break;
                }
                
                let end_term = std::cmp::min(start_term + chunk_size, terms_needed);
                let terms_in_chunk = end_term - start_term;
                
                if terms_in_chunk == 0 {
                    continue;
                }
                
                // 创建迭代器
                let mut iter = ChudnovskyIter::new(prec);
                
                // 跳过前面的项
                for _ in 0..start_term {
                    iter.next_term();
                }
                
                // 计算这个块的和
                let chunk_sum = iter.sum_next_n_terms(terms_in_chunk);
                local_sum += chunk_sum;
            }
            
            // 添加到全局结果
            let mut global_sum = result.lock().unwrap();
            *global_sum += &local_sum;
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
    
    // 计算 π = (426880 * sqrt(10005)) / sum
    let sqrt_10005 = Float::with_val(precision, 10005.0).sqrt();
    let numerator = Float::with_val(precision, 426880.0) * sqrt_10005;
    let pi = numerator / sum;
    
    let duration = start.elapsed().as_secs_f64();
    println!("计算完成，耗时: {:.2} 秒", duration);
    println!("平均速度: {:.1} 位/秒", digits as f64 / duration);
    
    (pi, duration)
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
    
    let duration = start.elapsed().as_secs_f64();
    println!("写入完成，耗时: {:.2} 秒", duration);
    
    Ok(())
}

// 内存使用统计
fn print_memory_stats(digits: usize, precision: u32, num_threads: usize) {
    println!("\n内存使用估算:");
    println!("{}", "-".repeat(40));
    
    let float_size_bytes = precision as f64 / 8.0;
    let thread_memory_mb = (num_threads as f64) * float_size_bytes / 1024.0 / 1024.0;
    let result_memory_mb = float_size_bytes / 1024.0 / 1024.0;
    let total_memory_mb = (num_threads as f64 + 1.0) * float_size_bytes / 1024.0 / 1024.0;
    
    println!("计算位数: {} 位十进制", digits);
    println!("精度: {} 位二进制", precision);
    println!("每个高精度浮点数: {:.2} MB", result_memory_mb);
    println!("线程内存: {:.2} MB ({} 个线程)", thread_memory_mb, num_threads);
    println!("总估算内存: {:.2} MB", total_memory_mb);
    
    if total_memory_mb > 100.0 {
        println!("⚠️  警告: 内存使用可能较高，考虑减少线程数或位数");
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

// 获取用户输入
fn get_user_input() -> (usize, usize) {
    println!("π 计算器 (Chudnovsky 算法)");
    println!("{}", "=".repeat(50));
    
    // 获取计算位数
    let digits = loop {
        print!("请输入要计算的 π 的位数 (1-1,000,000, 默认 1000): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            break 1000;
        }
        
        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= 1_000_000 => break n,
            Ok(_) => println!("位数必须在 1 到 1,000,000 之间"),
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

fn main() {
    let (digits, num_threads) = get_user_input();
    
    println!("\n{}", "=".repeat(50));
    println!("开始计算 π 到 {} 位有效数字", digits);
    println!("使用 {} 个线程", num_threads);
    println!("{}", "=".repeat(50));
    
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;
    print_memory_stats(digits, precision, num_threads);
    
    // 计算 π
    let (pi, compute_time) = compute_pi_chudnovsky(digits, num_threads);
    
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
    let filename = format!("pi_chudnovsky_{}_digits.txt", digits);
    println!("\n写入文件...");
    println!("{}", "-".repeat(52));
    
    match write_pi_to_file(&pi, digits, &filename) {
        Ok(_) => {
            if let Ok(metadata) = std::fs::metadata(&filename) {
                println!("\n文件信息:");
                println!("{}", "-".repeat(52));
                println!("文件名: {}", filename);
                println!("文件大小: {:.2} KB", metadata.len() as f64 / 1024.0);
                println!("计算时间: {:.2} 秒", compute_time);
                println!("平均速度: {:.2} 位/秒", digits as f64 / compute_time);
                
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
