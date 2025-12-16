use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use std::io::{self, Write};
use rug::{Float, Integer, Assign};
use rug::ops::Pow;
use num_cpus;

// 优化的 Chudnovsky 算法计算器
struct ChudnovskyCalculator {
    // 预分配的临时变量
    term: Float,
    numerator: Integer,
    denominator: Integer,
    k_factorial: Integer,
    three_k_factorial: Integer,
    six_k_factorial: Integer,
    // 常数
    c: Integer,
    d: Integer,
    sqrt_constant: Float,
    scale_factor: Float,
}

impl ChudnovskyCalculator {
    fn new(precision: u32) -> Self {
        let prec = precision;
        
        // 初始化常数
        let c = Integer::from(13591409);
        let d = Integer::from(545140134);
        
        // 计算 sqrt(10005) - 使用正确的 API
        let mut sqrt_10005 = Float::with_val(prec, 10005);
        sqrt_10005.sqrt_mut();
        let sqrt_constant = sqrt_10005.clone();
        
        let mut scale_factor = Float::with_val(prec, 426880);
        scale_factor *= &sqrt_constant;
        
        Self {
            term: Float::with_val(prec, 0),
            numerator: Integer::new(),
            denominator: Integer::new(),
            k_factorial: Integer::from(1),
            three_k_factorial: Integer::from(1),
            six_k_factorial: Integer::from(1),
            c,
            d,
            sqrt_constant,
            scale_factor,
        }
    }
    
    // 计算 Chudnovsky 算法的单项
    fn compute_term(&mut self, k: usize) -> &Float {
        if k == 0 {
            // k=0 的特殊情况
            self.numerator.assign(1);
            self.denominator.assign(1);
        } else {
            // 使用递推关系计算阶乘，避免重复计算
            self.update_factorials(k);
            
            // 计算分子: (-1)^k * (6k)! * (13591409 + 545140134k)
            self.numerator.assign(&self.six_k_factorial);
            let mut coefficient = Integer::from(&self.c);
            coefficient += &self.d * k;
            self.numerator *= &coefficient;
            
            if k % 2 == 1 {
                self.numerator = (-&self.numerator).into();
            }
            
            // 计算分母: (3k)! * (k^3 * 640320^(3k)
            self.denominator.assign(&self.three_k_factorial);
            let k_fact_cubed = Integer::from(&self.k_factorial).pow(3);
            self.denominator *= &k_fact_cubed;
            
            let base_640320 = Integer::from(640320);
            let exponent = (3 * k) as u32;
            let power_term = base_640320.pow(exponent);
            self.denominator *= &power_term;
        }
        
        // 将分数转换为浮点数
        let num_float = Float::with_val(self.term.prec(), &self.numerator);
        let den_float = Float::with_val(self.term.prec(), &self.denominator);
        
        self.term.assign(&num_float / &den_float);
        &self.term
    }
    
    // 使用递推关系更新阶乘
    fn update_factorials(&mut self, k: usize) {
        if k == 1 {
            self.k_factorial.assign(1);
            self.three_k_factorial.assign(6); // 3!
            self.six_k_factorial.assign(720); // 6!
            return;
        }
        
        // 递推计算阶乘
        // k! = (k-1)! * k
        self.k_factorial *= k;
        
        // (3k)! = (3(k-1))! * (3k-2)*(3k-1)*3k
        let three_k_minus_2 = 3 * k - 2;
        let three_k_minus_1 = 3 * k - 1;
        let three_k = 3 * k;
        
        self.three_k_factorial *= three_k_minus_2;
        self.three_k_factorial *= three_k_minus_1;
        self.three_k_factorial *= three_k;
        
        // (6k)! = (6(k-1))! * (6k-5)*(6k-4)*(6k-3)*(6k-2)*(6k-1)*6k
        for i in 1..=6 {
            let factor = 6 * k - 6 + i;
            self.six_k_factorial *= factor;
        }
    }
}

// 优化的并行 Chudnovsky 算法
fn compute_pi_chudnovsky(log10_digits: f64, num_threads: usize) -> (Float, f64) {
    // 计算实际位数: digits = 10^log10_digits
    let digits = 10f64.powf(log10_digits).round() as usize;
    let actual_log10 = (digits as f64).log10();
    
    println!("计算 π 到 10^{:.2} ≈ {} 位有效数字", actual_log10, digits);
    println!("使用 {} 个线程...", num_threads);
    
    let start = Instant::now();
    
    // 计算所需精度（二进制位）
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 32;
    
    // Chudnovsky 算法每项提供约 14 位十进制精度
    let terms_needed = (digits as f64 / 14.0).ceil() as usize + 2;
    
    println!("精度: {} 位二进制", precision);
    println!("需要计算 {} 项...", terms_needed);
    
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(num_threads);
    
    for _ in 0..num_threads {
        let counter = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            let mut calculator = ChudnovskyCalculator::new(precision);
            let mut local_sum = Float::with_val(precision, 0);
            
            loop {
                let k = counter.fetch_add(1, Ordering::SeqCst);
                if k >= terms_needed {
                    break;
                }
                
                let term = calculator.compute_term(k);
                local_sum += term;
            }
            
            local_sum
        });
        
        handles.push(handle);
    }
    
    // 收集并合并结果
    let mut series_sum = Float::with_val(precision, 0);
    for handle in handles {
        let thread_sum = handle.join().unwrap();
        series_sum += thread_sum;
    }
    
    // 计算最终结果: π = (426880 * sqrt(10005)) / series_sum
    let mut sqrt_10005 = Float::with_val(precision, 10005);
    sqrt_10005.sqrt_mut();
    let mut numerator = Float::with_val(precision, 426880);
    numerator *= &sqrt_10005;
    let pi = numerator / series_sum;
    
    let duration = start.elapsed().as_secs_f64();
    println!("计算完成，耗时: {:.2} 秒", duration);
    println!("平均速度: {:.2} 位/秒", digits as f64 / duration);
    
    (pi, duration)
}

// 性能优化的文件写入
fn write_pi_to_file_optimized(
    pi: &Float, 
    digits: usize, 
    filename: &str,
) -> io::Result<()> {
    println!("将结果写入文件 {}...", filename);
    let start = Instant::now();
    
    let file = std::fs::File::create(filename)?;
    let mut writer = io::BufWriter::new(file);
    
    // 写入头信息
    writeln!(writer, "π 的前 {} 位有效数字", digits)?;
    writeln!(writer, "计算时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(writer, "{}", "=".repeat(80))?;
    
    // 直接写入 π 值，避免字符串转换的内存开销
    write!(writer, "3.")?;
    
    // 逐位计算和写入，避免大字符串内存分配
    let mut remainder = Float::with_val(pi.prec(), pi);
    remainder -= 3; // 减去整数部分
    
    let ten = Float::with_val(pi.prec(), 10);
    
    for i in 0..digits {
        remainder *= &ten;
        let digit_int = remainder.to_integer().unwrap();
        let digit = digit_int.to_u32().unwrap() as u8;
        remainder -= digit;
        
        write!(writer, "{}", digit)?;
        
        // 格式化：每 50 位一行，每 10 位一组
        if (i + 1) % 50 == 0 {
            writeln!(writer)?;
        } else if (i + 1) % 10 == 0 {
            write!(writer, " ")?;
        }
        
        // 进度报告
        if (i + 1) % 1000 == 0 {
            println!("已写入 {} 位...", i + 1);
            writer.flush()?;
        }
    }
    
    // 写入统计信息
    writeln!(writer, "\n{}", "=".repeat(80))?;
    writeln!(writer, "统计信息:")?;
    writeln!(writer, "总位数: {}", digits)?;
    
    writer.flush()?;
    
    let duration = start.elapsed().as_secs_f64();
    println!("写入完成，耗时: {:.2} 秒", duration);
    
    if let Ok(metadata) = std::fs::metadata(filename) {
        println!("文件大小: {:.2} KB", metadata.len() as f64 / 1024.0);
    }
    
    Ok(())
}

// 优化的内存统计
fn print_optimized_memory_stats(log10_digits: f64, num_threads: usize) {
    let digits = 10f64.powf(log10_digits).round() as usize;
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 32;
    
    println!("\n内存使用估算:");
    println!("{}", "-".repeat(40));
    
    let float_size_bytes = (precision as f64) / 8.0;
    let thread_memory_mb = (num_threads as f64) * float_size_bytes / 1024.0 / 1024.0;
    let total_memory_mb = (num_threads as f64 + 2.0) * float_size_bytes / 1024.0 / 1024.0;
    
    println!("计算位数: 10^{:.2} ≈ {} 位", log10_digits, digits);
    println!("精度: {} 位二进制", precision);
    println!("每个高精度数: {:.2} MB", float_size_bytes / 1024.0 / 1024.0);
    println!("线程内存: {:.2} MB ({} 线程)", thread_memory_mb, num_threads);
    println!("总估算内存: {:.2} MB", total_memory_mb);
    
    if total_memory_mb > 500.0 {
        println!("⚠️  建议使用更多线程减少每线程内存");
    }
}

// 优化的输入获取
fn get_optimized_input() -> (f64, usize, String) {
    println!("π 计算器 (优化版 - Chudnovsky 算法)");
    println!("{}", "=".repeat(50));
    
    let max_threads = num_cpus::get();
    let default_log10 = 3.0; // 默认 1000 位
    
    // 获取 log10(位数)
    let log10_digits = loop {
        print!("请输入所需位数的对数 (log10), 例如 3 表示 10^3=1000 位 (默认 {:.1}): ", default_log10);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            break default_log10;
        }
        
        match input.parse::<f64>() {
            Ok(n) => {
                if n >= 1.0 && n <= 6.0 {
                    break n;
                } else if n < 1.0 {
                    println!("对数必须 ≥ 1.0 (至少 10 位)");
                } else {
                    println!("对数必须 ≤ 6.0 (最多 1,000,000 位)");
                }
            }
            Err(_) => println!("请输入有效的数字"),
        }
    };
    
    let digits = 10f64.powf(log10_digits).round() as usize;
    
    // 根据位数自动选择线程数
    let auto_threads = if digits <= 1000 {
        std::cmp::max(1, max_threads / 2) // 小计算用较少线程
    } else {
        max_threads // 大计算用所有线程
    };
    
    let num_threads = loop {
        print!("请输入线程数 (1-{}, 默认 {}): ", max_threads, auto_threads);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            break auto_threads;
        }
        
        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= max_threads => break n,
            Ok(_) => println!("线程数必须在 1 到 {} 之间", max_threads),
            Err(_) => println!("请输入有效的数字"),
        }
    };
    
    let filename = format!("pi_10pow{:.1}_digits.txt", log10_digits);
    let output_file = loop {
        print!("请输入输出文件名 (默认 {}): ", filename);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            break filename;
        } else {
            break input.to_string();
        }
    };
    
    (log10_digits, num_threads, output_file)
}

// 性能分析函数
fn analyze_performance(log10_digits: f64, compute_time: f64, digits: usize) {
    println!("\n性能分析:");
    println!("{}", "-".repeat(40));
    
    let speed = digits as f64 / compute_time;
    println!("计算速度: {:.2} 位/秒", speed);
    
    if speed < 1000.0 {
        println!("性能等级: 较慢 - 考虑减少位数或增加线程");
    } else if speed < 10000.0 {
        println!("性能等级: 中等");
    } else if speed < 100000.0 {
        println!("性能等级: 快速");
    } else {
        println!("性能等级: 极速");
    }
    
    // 预测更大规模的计算时间
    if log10_digits < 6.0 {
        let next_log10 = log10_digits + 1.0;
        let predicted_time = compute_time * 10.0; // Chudnovsky 算法复杂度接近线性
        
        println!("预测 10^{:.1} 位计算时间: {:.1} 秒", next_log10, predicted_time);
    }
}

fn main() {
    let (log10_digits, num_threads, output_file) = get_optimized_input();
    let digits = 10f64.powf(log10_digits).round() as usize;
    
    println!("\n{}", "=".repeat(50));
    println!("开始计算 π 到 10^{:.2} ≈ {} 位有效数字", log10_digits, digits);
    println!("使用 {} 个线程", num_threads);
    println!("输出文件: {}", output_file);
    println!("算法: Chudnovsky (每项提供约 14 位精度)");
    println!("{}", "=".repeat(50));
    
    print_optimized_memory_stats(log10_digits, num_threads);
    
    // 计算 π
    let (pi, compute_time) = compute_pi_chudnovsky(log10_digits, num_threads);
    
    // 显示预览
    println!("\nπ 的前 50 位:");
    println!("{}", "-".repeat(52));
    
    // 使用逐位计算显示预览，避免大字符串转换
    print!("3.");
    let mut remainder = Float::with_val(pi.prec(), &pi);
    remainder -= 3;
    let ten = Float::with_val(pi.prec(), 10);
    
    for i in 0..50 {
        remainder *= &ten;
        let digit_int = remainder.to_integer().unwrap();
        let digit = digit_int.to_u32().unwrap();
        remainder -= digit;
        print!("{}", digit);
        
        if (i + 1) % 10 == 0 && i < 49 {
            print!(" ");
        }
    }
    println!();
    
    // 性能分析
    analyze_performance(log10_digits, compute_time, digits);
    
    // 写入文件
    println!("\n写入文件...");
    println!("{}", "-".repeat(52));
    
    match write_pi_to_file_optimized(&pi, digits, &output_file) {
        Ok(_) => {
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                println!("\n文件信息:");
                println!("{}", "-".repeat(52));
                println!("文件名: {}", output_file);
                println!("文件大小: {:.2} KB", metadata.len() as f64 / 1024.0);
                println!("计算时间: {:.2} 秒", compute_time);
                
                if metadata.len() < 1024 * 1024 {
                    println!("\n提示: 使用 'more {}' 或 'head -n 50 {}' 查看内容", output_file, output_file);
                } else {
                    println!("\n提示: 文件较大，建议使用 'head -n 50 {}' 查看开头", output_file);
                }
            }
        }
        Err(e) => {
            eprintln!("写入文件失败: {}", e);
        }
    }
    
    println!("\n计算完成！结果已保存到 {}", output_file);
}
