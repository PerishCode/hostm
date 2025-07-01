use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use chrono::Local;

#[derive(Parser)]
#[command(name = "hostm")]
#[command(about = "管理 /etc/hosts 文件的工具")]
#[command(version)]
#[command(propagate_version = true)]
struct Args {
    /// hosts 文件路径，默认为 /etc/hosts
    #[arg(short = 'f', long, default_value = "/etc/hosts")]
    hosts_file: String,

    /// 输出详细日志
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 更新已存在的域名映射
    Update {
        /// 域名
        domain: String,
        /// 新的 IP 地址
        ip: String,
    },
    /// 删除域名映射
    Delete {
        /// 要删除的域名
        domain: String,
    },
    /// 创建新的域名映射
    Create {
        /// 域名
        domain: String,
        /// IP 地址
        ip: String,
    },
    /// 查找域名映射
    Search {
        /// 要查找的域名（支持部分匹配）
        domain: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match &args.command {
        Commands::Update { domain, ip } => {
            update_domain(domain, ip, &args.hosts_file, args.verbose)
        }
        Commands::Delete { domain } => {
            delete_domain(domain, &args.hosts_file, args.verbose)
        }
        Commands::Create { domain, ip } => {
            create_domain(domain, ip, &args.hosts_file, args.verbose)
        }
        Commands::Search { domain } => {
            search_domain(domain, &args.hosts_file, args.verbose)
        }
    }
}

/// 更新已存在的域名映射
fn update_domain(domain: &str, ip: &str, hosts_file: &str, verbose: bool) -> Result<()> {
    let hosts_path = Path::new(hosts_file);
    
    // 检查文件
    check_hosts_file(hosts_path)?;
    
    // 读取文件内容
    let content = fs::read_to_string(hosts_path)
        .with_context(|| format!("无法读取文件: {}", hosts_file))?;
    
    if verbose {
        println!("[verbose] 更新域名映射: {} -> {}", domain, ip);
    }
    
    let new_content = update_existing_domain(&content, domain, ip, verbose)?;
    
    // 写入文件
    write_hosts_file(hosts_path, &new_content, hosts_file, verbose)?;
    
    println!("✅ 已更新域名映射: {} -> {}", domain, ip);
    Ok(())
}

/// 删除域名映射
fn delete_domain(domain: &str, hosts_file: &str, verbose: bool) -> Result<()> {
    let hosts_path = Path::new(hosts_file);
    
    // 检查文件
    check_hosts_file(hosts_path)?;
    
    // 读取文件内容
    let content = fs::read_to_string(hosts_path)
        .with_context(|| format!("无法读取文件: {}", hosts_file))?;
    
    if verbose {
        println!("[verbose] 删除域名: {}", domain);
    }
    
    let new_content = remove_domain(&content, domain, verbose)?;
    
    // 写入文件
    write_hosts_file(hosts_path, &new_content, hosts_file, verbose)?;
    
    println!("✅ 已删除域名映射: {}", domain);
    Ok(())
}

/// 创建新的域名映射
fn create_domain(domain: &str, ip: &str, hosts_file: &str, verbose: bool) -> Result<()> {
    let hosts_path = Path::new(hosts_file);
    
    // 检查文件
    check_hosts_file(hosts_path)?;
    
    // 读取文件内容
    let content = fs::read_to_string(hosts_path)
        .with_context(|| format!("无法读取文件: {}", hosts_file))?;
    
    if verbose {
        println!("[verbose] 创建域名映射: {} -> {}", domain, ip);
    }
    
    let new_content = add_new_domain(&content, domain, ip, verbose)?;
    
    // 写入文件
    write_hosts_file(hosts_path, &new_content, hosts_file, verbose)?;
    
    println!("✅ 已创建域名映射: {} -> {}", domain, ip);
    Ok(())
}

/// 查找域名映射
fn search_domain(domain: &str, hosts_file: &str, verbose: bool) -> Result<()> {
    let hosts_path = Path::new(hosts_file);
    
    // 检查文件
    check_hosts_file(hosts_path)?;
    
    // 读取文件内容
    let content = fs::read_to_string(hosts_path)
        .with_context(|| format!("无法读取文件: {}", hosts_file))?;
    
    if verbose {
        println!("[verbose] 查找包含 '{}' 的行", domain);
    }
    
    let mut found = false;
    for (line_num, line) in content.lines().enumerate() {
        if line.contains(domain) {
            if !found {
                println!("🔍 找到包含 '{}' 的行:", domain);
                found = true;
            }
            println!("  {}: {}", line_num + 1, line);
        }
    }
    
    if !found {
        println!("❌ 未找到包含 '{}' 的行", domain);
    }
    
    Ok(())
}

/// 检查 hosts 文件
fn check_hosts_file(hosts_path: &Path) -> Result<()> {
    if !hosts_path.exists() {
        anyhow::bail!("hosts 文件不存在: {}", hosts_path.display());
    }
    
    if !hosts_path.is_file() {
        anyhow::bail!("路径不是文件: {}", hosts_path.display());
    }
    
    Ok(())
}

/// 写入 hosts 文件
fn write_hosts_file(hosts_path: &Path, content: &str, hosts_file: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("[verbose] 写入 hosts 文件: {}", hosts_file);
    }
    
    match fs::write(hosts_path, content) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                anyhow::bail!("权限不足，无法写入文件: {}", hosts_file);
            } else {
                Err(e).with_context(|| format!("无法写入文件: {}", hosts_file))?
            }
        }
    }
}

/// 更新已存在的域名映射
fn update_existing_domain(content: &str, domain: &str, ip: &str, verbose: bool) -> Result<String> {
    let ip_regex = Regex::new(r"^([0-9]+\.){3}[0-9]+[[:space:]]+")?;
    let domain_regex = Regex::new(&format!(r"\b{}\b", regex::escape(domain)))?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut domain_found = false;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let comment = format!("# updated by hostm {}", timestamp);
    
    // 查找并更新现有域名
    for line in &mut lines {
        if ip_regex.is_match(line) && domain_regex.is_match(line) {
            if verbose {
                println!("[verbose] 更新行: {} => {} {} {}", line, ip, domain, comment);
            }
            *line = format!("{} {} {}", ip, domain, comment);
            domain_found = true;
            break;
        }
    }
    
    if !domain_found {
        anyhow::bail!("域名 '{}' 不存在，请使用 'create' 命令创建新映射", domain);
    }
    
    let result = lines.join("\n");
    Ok(result + if content.ends_with('\n') { "\n" } else { "" })
}

/// 删除域名映射
fn remove_domain(content: &str, domain: &str, verbose: bool) -> Result<String> {
    let ip_regex = Regex::new(r"^([0-9]+\.){3}[0-9]+[[:space:]]+")?;
    let domain_regex = Regex::new(&format!(r"\b{}\b", regex::escape(domain)))?;
    
    let mut found = false;
    let lines: Vec<&str> = content.lines()
        .filter(|line| {
            let matched = ip_regex.is_match(line) && domain_regex.is_match(line);
            if verbose && matched {
                println!("[verbose] 删除行: {}", line);
                found = true;
            }
            !matched
        })
        .collect();
    
    if !found {
        anyhow::bail!("域名 '{}' 不存在，无需删除", domain);
    }
    
    Ok(lines.join("\n") + if content.ends_with('\n') { "\n" } else { "" })
}

/// 添加新的域名映射
fn add_new_domain(content: &str, domain: &str, ip: &str, verbose: bool) -> Result<String> {
    let ip_regex = Regex::new(r"^([0-9]+\.){3}[0-9]+[[:space:]]+")?;
    let domain_regex = Regex::new(&format!(r"\b{}\b", regex::escape(domain)))?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let comment = format!("# created by hostm {}", timestamp);
    
    // 检查域名是否已存在
    for line in &lines {
        if ip_regex.is_match(line) && domain_regex.is_match(line) {
            anyhow::bail!("域名 '{}' 已存在，请使用 'update' 命令更新", domain);
        }
    }
    
    // 添加新行
    if verbose {
        println!("[verbose] 添加新行: {} {} {}", ip, domain, comment);
    }
    lines.push(format!("{} {} {}", ip, domain, comment));
    
    let result = lines.join("\n");
    Ok(result + if content.ends_with('\n') { "\n" } else { "" })
}
