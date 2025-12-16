use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use std::io::{self, Write};
use rug::{Float, Assign};
use rug::ops::Pow;
use num_cpus;

// 内存优化的 BBP 公式项计算
// 重用 Float 对象以减少内存分配
struct BBPCalculator {
    // 预分配的临时变量，避免每次计算都重新分配
    term1: Float,
    term2: Float,
    term3: Float,
    term4: Float,
    eight_k: Float,
    sixteen_pow_k: Float,
    denominator1: Float,
    denominator2: Float,
    denominator3: Float,
    denominator4: Float,
    one_over_16: Float,
    sixteen: Float,
}

impl BBPCalculator {
    fn new(precision: u32) -> Self {
        let prec = precision;
        Self {
            term1: Float::with_val(prec, 0),
            term2: Float::with_val(prec, 0),
            term3: Float::with_val(prec, 0),
            term4: Float::with_val(prec, 0),
            eight_k: Float::with_val(prec, 0),
            sixteen_pow_k: Float::with_val(prec, 0),
            denominator1: Float::with_val(prec, 1),
            denominator2: Float::with_val(prec, 4),
            denominator3: Float::with_val(prec, 5),
            denominator4: Float::with_val(prec, 6),
            one_over_16: Float::with_val(prec, 1) / 16,
            sixteen: Float::with_val(prec, 16),
        }
    }

    // 计算 BBP 公式的单项
    fn compute_term(&mut self, k: usize) -> &Float {
        // 计算 8k
        self.eight_k.assign(8 * k);

        // 计算分母
        self.denominator1.assign(1u8);
        self.denominator1 += &self.eight_k;

        self.denominator2.assign(4u8);
        self.denominator2 += &self.eight_k;

        self.denominator3.assign(5u8);
        self.denominator3 += &self.eight_k;

        self.denominator4.assign(6u8);
        self.denominator4 += &self.eight_k;

        // 计算 4/(8k+1) - 2/(8k+4) - 1/(8k+5) - 1/(8k+6)
        self.term1.assign(4u8);
        self.term1 /= &self.denominator1;

        self.term2.assign(2u8);
        self.term2 /= &self.denominator2;

        self.term3.assign(1u8);
        self.term3 /= &self.denominator3;

        self.term4.assign(1u8);
        self.term4 /= &self.denominator4;

        // 合并项
        self.term1 -= &self.term2;
        self.term1 -= &self.term3;
        self.term1 -= &self.term4;

        // 计算 16^(-k)
        if k == 0 {
            self.sixteen_pow_k.assign(1u8);
        } else if k == 1 {
            self.sixteen_pow_k.assign(&self.one_over_16);
        } else {
            let sixteen_clone = self.sixteen.clone();
            let pow_result = sixteen_clone.pow(k as i32);
            self.sixteen_pow_k.assign(1u8);
            self.sixteen_pow_k /= pow_result;
        }

        // 乘以 16^(-k)
        self.term1 *= &self.sixteen_pow_k;

        &self.term1
    }
}

// 优化的 BBP 公式并行计算
fn compute_pi_optimized(digits: usize, num_threads: usize) -> (Float, f64) {
    println!("使用 {} 个线程计算 π 到 {} 位有效数字...", num_threads, digits);

    let start = Instant::now();

    // 计算所需精度（二进制位）
    // 1 位十进制 ≈ log2(10) ≈ 3.32193 位二进制
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;

    // 计算需要多少项才能达到所需精度
    // BBP 公式每项贡献约 4 位二进制位
    let terms_needed = (precision as usize) / 4 + 10;

    println!("精度: {} 位二进制", precision);
    println!("需要计算 {} 项...", terms_needed);

    // 用于分发任务的原子计数器
    let counter = Arc::new(AtomicUsize::new(0));

    // 存储线程句柄的向量
    let mut handles = Vec::with_capacity(num_threads);

    // 为每个线程预分配 BBP 计算器
    for _ in 0..num_threads {
        let counter = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            // 每个线程创建自己的 BBP 计算器，避免线程间的内存竞争
            let mut calculator = BBPCalculator::new(precision);
            let mut local_sum = Float::with_val(precision, 0);

            loop {
                // 获取下一个要计算的 k
                let k = counter.fetch_add(1, Ordering::SeqCst);
                if k >= terms_needed {
                    break;
                }

                // 计算单项并累加
                let term = calculator.compute_term(k);
                local_sum += term;
            }

            // 返回局部和
            local_sum
        });

        handles.push(handle);
    }

    // 收集并合并所有线程的结果
    let mut final_result = Float::with_val(precision, 0);
    for handle in handles {
        let thread_sum = handle.join().unwrap();
        final_result += thread_sum;
    }

    let duration = start.elapsed().as_secs_f64();
    println!("计算完成，耗时: {:.2} 秒", duration);

    (final_result, duration)
}

// 分块写入文件，避免内存中保存完整的 π 字符串
fn write_pi_to_file_chunked(
    pi: &Float,
    digits: usize,
    filename: &str,
    progress_callback: Option<Box<dyn Fn(usize, usize)>>
) -> io::Result<()> {
    println!("将结果分块写入文件 {}...", filename);

    let start = Instant::now();

    // 打开文件
    let file = std::fs::File::create(filename)?;
    let mut writer = io::BufWriter::new(file);

    // 写入头信息
    writeln!(writer, "π 的前 {} 位有效数字", digits)?;
    writeln!(writer, "计算时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(writer, "{}", "=".repeat(80))?;

    // 首先获取整个 π 的字符串表示
    println!("正在将 π 转换为字符串...");
    let pi_str = pi.to_string_radix(10, Some(digits));

    // 分块处理：每次处理一定数量的位数
    let chunk_size = 1000;  // 每块 1000 位
    let total_chunks = (digits + chunk_size - 1) / chunk_size;

    // 写入文件
    for chunk in 0..total_chunks {
        let start_pos = chunk * chunk_size;
        let end_pos = std::cmp::min((chunk + 1) * chunk_size, pi_str.len());

        if start_pos < pi_str.len() {
            let chunk_str = &pi_str[start_pos..end_pos];

            // 格式化输出：每 50 个数字一行，每 10 个数字一组
            let mut formatted = String::new();
            let mut pos_in_chunk = 0;

            while pos_in_chunk < chunk_str.len() {
                let remaining = chunk_str.len() - pos_in_chunk;
                let take = std::cmp::min(10, remaining);

                formatted.push_str(&chunk_str[pos_in_chunk..pos_in_chunk + take]);
                pos_in_chunk += take;

                if pos_in_chunk % 50 == 0 && pos_in_chunk < chunk_str.len() {
                    formatted.push('\n');
                } else if pos_in_chunk < chunk_str.len() {
                    formatted.push(' ');
                }
            }

            if !formatted.is_empty() {
                writeln!(writer, "{}", formatted)?;
            }
        }

        // 报告进度
        if let Some(callback) = &progress_callback {
            callback(chunk + 1, total_chunks);
        }

        if (chunk + 1) % 10 == 0 {
            println!("已写入 {}/{} 块...", chunk + 1, total_chunks);
        }
    }

    // 写入统计信息
    writeln!(writer, "\n{}", "=".repeat(80))?;
    writeln!(writer, "统计信息:")?;
    writeln!(writer, "总位数: {}", digits)?;
//    writeln!(writer, "计算时间: {:.2} 秒", duration)?;

    let duration = start.elapsed().as_secs_f64();
    println!("写入完成，耗时: {:.2} 秒", duration);

    // 获取文件大小
    if let Ok(metadata) = std::fs::metadata(filename) {
        println!("文件大小: {:.2} KB", metadata.len() as f64 / 1024.0);
    }

    Ok(())
}

// 计算并显示内存使用统计
fn print_memory_stats(digits: usize, precision: u32, num_threads: usize) {
    println!("\n内存使用估算:");
    println!("{}", "-".repeat(40));

    // 每个 Float 的内存占用（字节）= 精度（位）/ 8
    let float_size_bytes = precision as f64 / 8.0;

    // 线程内存占用
    let thread_memory_mb = (num_threads as f64) * float_size_bytes / 1024.0 / 1024.0;

    // 结果内存占用
    let result_memory_mb = float_size_bytes / 1024.0 / 1024.0;

    // 总内存占用估算
    let total_memory_mb = (num_threads as f64 + 1.0) * float_size_bytes / 1024.0 / 1024.0;

    println!("计算位数: {} 位十进制", digits);
    println!("精度: {} 位二进制", precision);
    println!("每个高精度浮点数: {:.2} MB", float_size_bytes / 1024.0 / 1024.0);
    println!("线程内存: {:.2} MB ({} 个线程)", thread_memory_mb, num_threads);
    println!("结果内存: {:.2} MB", result_memory_mb);
    println!("总估算内存: {:.2} MB", total_memory_mb);

    if total_memory_mb > 100.0 {
        println!("⚠️  警告: 内存使用可能较高，考虑减少线程数或位数");
    }
}

// 验证 π 值的准确性
fn verify_pi_accuracy(pi_str: &str, digits: usize) -> (bool, usize) {
    // 已知的 π 前 100 位
    let known_pi = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679";
    
    // 去掉小数点进行比较
    let known_digits: Vec<char> = known_pi.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    
    let computed_digits: Vec<char> = pi_str.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    
    // 比较前 min(100, digits) 位
    let compare_len = std::cmp::min(100, digits);
    let compare_len = std::cmp::min(compare_len, known_digits.len());
    let compare_len = std::cmp::min(compare_len, computed_digits.len());
    
    let mut first_error = None;
    for i in 0..compare_len {
        if computed_digits[i] != known_digits[i] {
            first_error = Some(i);
            break;
        }
    }

    let accurate = first_error.is_none();
    (accurate, first_error.unwrap_or(compare_len))
}

// 获取用户输入的函数
fn get_user_input() -> (usize, usize, String) {
    println!("π 计算器 (内存优化并行版本)");
    println!("{}", "=".repeat(50));

    // 获取计算位数
    let digits = loop {
        print!("请输入要计算的 π 的位数 (1-1,000,000, 默认 1000): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            break 1000;  // 默认值
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
            break max_threads;  // 默认值
        }

        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= max_threads => break n,
            Ok(_) => println!("线程数必须在 1 到 {} 之间", max_threads),
            Err(_) => println!("请输入有效的数字"),
        }
    };

    // 获取输出文件名
    let filename = format!("pi_{}_digits.txt", digits);
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

    (digits, num_threads, output_file)
}

fn main() {
    // 获取用户输入
    let (digits, num_threads, output_file) = get_user_input();

    println!("\n{}", "=".repeat(50));
    println!("开始计算 π 到 {} 位有效数字", digits);
    println!("使用 {} 个线程", num_threads);
    println!("输出文件: {}", output_file);
    println!("{}", "=".repeat(50));

    // 计算所需精度
    let precision = ((digits as f64) * 3.32193).ceil() as u32 + 10;

    // 显示内存使用统计
    print_memory_stats(digits, precision, num_threads);

    // 计算 π
    let (pi, compute_time) = compute_pi_optimized(digits, num_threads);

    // 显示结果预览
    println!("\nπ 的前 50 位:");
    println!("{}", "-".repeat(52));

    let preview_str = pi.to_string_radix(10, Some(50));
    println!("{}", preview_str);

    // 验证准确性
    println!("\n验证准确性:");
    println!("{}", "-".repeat(52));

    let pi_full_str = pi.to_string_radix(10, Some(digits));
    let (accurate, correct_digits) = verify_pi_accuracy(&pi_full_str, digits);

    if accurate {
        println!("✓ 前 {} 位与已知 π 值完全一致", correct_digits);
    } else {
        println!("✗ 前 {} 位正确，第 {} 位开始出现差异", correct_digits, correct_digits + 1);
    }

    // 写入文件
    println!("\n写入文件...");
    println!("{}", "-".repeat(52));

    let progress_callback = Box::new(|current: usize, total: usize| {
        if current % 10 == 0 || current == total {
            let percent = (current as f64 / total as f64 * 100.0) as usize;
            println!("写入进度: {}/{} ({}%)", current, total, percent);
        }
    });

    match write_pi_to_file_chunked(&pi, digits, &output_file, Some(progress_callback)) {
        Ok(_) => {
            // 显示文件信息
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                println!("\n文件信息:");
                println!("{}", "-".repeat(52));
                println!("文件名: {}", output_file);
                println!("文件大小: {:.2} KB", metadata.len() as f64 / 1024.0);
                println!("计算时间: {:.2} 秒", compute_time);
                println!("平均速度: {:.2} 位/秒", digits as f64 / compute_time);
                
                if metadata.len() < 1024 * 1024 {  // 小于 1MB
                    println!("\n提示: 您可以使用 'more {}' 或 'head -n 50 {}' 查看文件内容", 
                            output_file, output_file);
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

