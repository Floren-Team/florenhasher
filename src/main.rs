use std::env;
use std::fs;
use std::path::Path;
use std::io::Read;

// Используем трейты для каждого алгоритма отдельно, чтобы избежать конфликтов
use md5::Digest as DigestMd5;
use sha1::Digest as DigestSha1;
use sha2::Digest as DigestSha2;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const NC: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Использование: florenhasher <алгоритм> <файл> ИЛИ scanhash onefile <алг> <файл> ИЛИ scanhash scanall");
        return;
    }

    match args[1].as_str() {
        "scanall" => scan_all(),
        "onefile" => {
            if args.len() < 4 { println!("Использование: scanhash onefile <алг> <файл>"); return; }
            check_hash(&args[2].to_lowercase(), &args[3]);
        }
        algo => {
            if args.len() < 3 { println!("Ошибка аргументов"); return; }
            check_hash(&algo.to_lowercase(), &args[2]);
        }
    }
}

fn check_hash(algo: &str, file: &str) {
    let sum_file = format!("{}.{}", file, algo);
    let algo_upper = algo.to_uppercase();

    if !Path::new(file).exists() || !Path::new(&sum_file).exists() {
        println!("  [SKIP] [{}] {} (Файл или контрольная сумма отсутствуют)", algo_upper, file);
        return;
    }

    let content = fs::read_to_string(&sum_file).unwrap_or_default();
    let expected_hash = content.split_whitespace().next().unwrap_or("").to_lowercase();
    let actual_hash = compute_hash(file, algo);
    let rid = extract_rid(&content);

    if actual_hash == expected_hash {
        if let Some(r) = rid {
            if is_expired(&r) {
                println!("  [{RED}EXPIRED{NC}] [{algo_upper}] {file} (Хеш верный, но RID {r} устарел!)");
            } else {
                println!("  [{GREEN}OK{NC}] [{algo_upper}] {file} (Хеш верный, RID: {r})");
            }
        } else {
            println!("  [{YELLOW}OK{NC}] [{algo_upper}] {file} (Хеш верный, {YELLOW}RID: Отсутствует{NC})");
        }
    } else {
        println!("  [{RED}FAIL{NC}] [{algo_upper}] {file} (Ошибка хеша! Ожидалось: {expected_hash}, получено: {actual_hash})");
    }
}

fn compute_hash(file_path: &str, algo: &str) -> String {
    let mut file = fs::File::open(file_path).expect("Не удалось открыть файл");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Не удалось прочитать файл");

    match algo {
        "md5"    => format!("{:x}", md5::compute(&buffer)),
        "sha1"   => format!("{:x}", sha1::Sha1::digest(&buffer)),
        "sha256" => format!("{:x}", sha2::Sha256::digest(&buffer)),
        "sha512" => format!("{:x}", sha2::Sha512::digest(&buffer)),
        _ => "неподдерживаемый_алгоритм".to_string(),
    }
}

fn extract_rid(content: &str) -> Option<String> {
    let start = content.find("[RID:")?;
    let end = content[start..].find(']')?;
    Some(content[start + 5..start + end].to_string())
}

fn is_expired(rid: &str) -> bool {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    rid.len() >= 8 && &rid[0..8] != today
}

fn scan_all() {
    println!("Запуск полной проверки...");
    let paths = fs::read_dir(".").unwrap();
    for path in paths.filter_map(|e| e.ok()) {
        let name = path.file_name().into_string().unwrap();
        if name.ends_with(".md5") || name.ends_with(".sha1") || name.ends_with(".sha256") || name.ends_with(".sha512") {
            let parts: Vec<&str> = name.split('.').collect();
            let algo = parts.last().unwrap();
            let base = parts[..parts.len() - 1].join(".");
            check_hash(algo, &base);
        }
    }
}
