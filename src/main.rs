use chrono::{DateTime, Duration, Local, TimeZone};
use clap::Parser;
use plotters::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Nginx日志文件路径
    #[arg(short, long)]
    log_file: PathBuf,

    /// 统计时间间隔（秒），如1表示每秒，300表示每5分钟
    #[arg(short, long, default_value_t = 1)]
    interval: u64,

    /// 输出图表文件名
    #[arg(short, long, default_value = "qps_chart.png")]
    output: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // 读取并解析日志文件
    let (timestamps, time_min, time_max) = parse_nginx_log(&args.log_file)?;

    // 按指定时间间隔统计QPS
    let stats = calculate_qps(&timestamps, args.interval, time_min, time_max);

    // 生成QPS图表
    generate_qps_chart(&stats, &args.output, args.interval, &args.log_file)?;

    println!("QPS图表已保存为 {}", args.output);
    Ok(())
}

/// 读取并解析日志文件
fn parse_nginx_log(
    log_path: &PathBuf,
) -> Result<(Vec<DateTime<Local>>, DateTime<Local>, DateTime<Local>), Box<dyn Error>> {
    println!("正在解析日志文件: {:?}", log_path);

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);

    let mut timestamps = Vec::new();
    let mut time_min = Local::now();
    let mut time_max = Local.timestamp_opt(0, 0).unwrap();

    // Nginx日志格式正则表达式
    // 示例: 47.100.64.252 - - [18/Mar/2025:00:00:04 +0800] "GET / HTTP/1.1" 404 236 "-" "Mozilla/5.0 ..."
    let re = Regex::new(r#"\[(\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2} [+-]\d{4})\]"#)?;

    for line in reader.lines() {
        let line = line?;
        if let Some(cap) = re.captures(&line) {
            if let Some(time_str) = cap.get(1) {
                // 解析时间格式 "18/Mar/2025:00:00:04 +0800"
                let time_str = time_str.as_str();
                if let Ok(dt) = DateTime::parse_from_str(time_str, "%d/%b/%Y:%H:%M:%S %z") {
                    let timestamp = dt.with_timezone(&Local);
                    timestamps.push(timestamp);

                    // 更新最小和最大时间
                    if timestamp < time_min {
                        time_min = timestamp;
                    }
                    if timestamp > time_max {
                        time_max = timestamp;
                    }
                }
            }
        }
    }

    println!("共解析 {} 条日志记录", timestamps.len());
    println!("时间范围: {} 到 {}", time_min, time_max);

    Ok((timestamps, time_min, time_max))
}

/// 按指定时间间隔统计QPS
fn calculate_qps(
    timestamps: &[DateTime<Local>],
    interval: u64,
    time_min: DateTime<Local>,
    time_max: DateTime<Local>,
) -> Vec<(DateTime<Local>, f64)> {
    println!("正在按 {} 秒间隔统计QPS...", interval);

    let mut counter = HashMap::new();
    let interval_seconds = Duration::seconds(interval as i64);

    // 确保所有时间间隔都初始化为0
    let mut current = time_min;
    while current <= time_max {
        counter.insert(current, 0);
        current = current + interval_seconds;
    }

    // 统计每个时间间隔内的请求数
    for &ts in timestamps {
        // 确定请求属于哪个时间间隔
        let diff_seconds = (ts - time_min).num_seconds();
        let interval_index = diff_seconds / interval_seconds.num_seconds();
        let interval_start =
            time_min + Duration::seconds(interval_index * interval_seconds.num_seconds());

        // 增加该时间间隔的请求计数
        if let Some(count) = counter.get_mut(&interval_start) {
            *count += 1;
        }
    }

    // 转换为QPS并排序
    let mut qps_data: Vec<(DateTime<Local>, f64)> = counter
        .iter()
        .map(|(&time, &count)| (time, count as f64))
        .collect();

    qps_data.sort_by_key(|&(time, _)| time);

    println!("统计完成，共 {} 个数据点", qps_data.len());
    qps_data
}

/// 生成QPS图表
fn generate_qps_chart(
    qps_data: &[(DateTime<Local>, f64)],
    output: &str,
    interval: u64,
    log_path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    println!("正在生成QPS图表...");

    if qps_data.is_empty() {
        return Err("没有有效的QPS数据".into());
    }

    // 确定图表大小
    let root = BitMapBackend::new(output, (1800, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // 确定Y轴范围
    let max_qps = qps_data
        .iter()
        .map(|&(_, qps)| qps)
        .fold(0.0 / 0.0, f64::max);
    let y_max = (max_qps * 1.1).max(1.0); // 最大值上浮10%，至少为1

    // 确定X轴标签格式
    let time_format = if interval < 60 {
        "%H:%M:%S"
    } else if interval < 3600 {
        "%H:%M"
    } else {
        "%m-%d %H:%M"
    };

    // 确定X轴范围
    let time_min = qps_data.first().unwrap().0;
    let time_max = qps_data.last().unwrap().0;

    // 创建图表
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} Nginx 请求数统计 ({}秒间隔)",
                log_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件"),
                interval
            ),
            ("SimSun", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(time_min..time_max, 0.0..y_max)?;

    // 配置网格和坐标轴
    chart
        .configure_mesh()
        .x_labels(8) // X轴标签数量
        .x_label_formatter(&|x| x.format(time_format).to_string())
        .y_desc("请求数")
        .axis_desc_style(("SimSun", 15))
        .draw()?;

    // 绘制QPS曲线
    chart
        .draw_series(LineSeries::new(
            qps_data.iter().map(|&(time, qps)| (time, qps)),
            &BLUE.mix(0.8),
        ))?
        .label("请求数")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // 添加数据点
    chart.draw_series(
        qps_data
            .iter()
            .map(|&(time, qps)| Circle::new((time, qps), 2, BLUE.filled())),
    )?;

    // 添加图例
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}
