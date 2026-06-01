use std::env;
use std::fs;
use std::path::Path;
use std::io::Read;
use sha1::Digest as DigestSha1;
use sha2::Digest as DigestSha2;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const NC: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Проверка аргументов
    if args.len() < 2 {
        eprintln!("Использование: florenhasher <действие> [файл] [алгоритм]");
        return;
    }

    let action = args[1].as_str();

    match action {
        "scanall" => scan_all(),
        _ => {
            // Если действие не scanall, ожидаем имя файла
            if args.len() < 3 {
                eprintln!("Ошибка: Укажите имя файла для проверки.");
                return;
            }
            let file = &args[2];
            
            // Проверка: имя файла начинается с "floren"
            if !file.to_lowercase().starts_with("floren") {
                eprintln!("{}Ошибка: Данный файл не является Floren Team скриптом.{}\nОзнакомиться: https://github.com/Floren-Team/florenhasher/blob/main/README.md", RED, NC);
                return;
            }

            // Если указан алгоритм (4-й аргумент), берем его, иначе ошибка
            if args.len() < 4 {
                eprintln!("Ошибка: Алгоритм не указан (md5, sha1, sha256, sha512).");
                return;
            }
            let algo = args[3].to_lowercase();
            check_hash(&algo, file);
        }
    }
}

fn check_hash(algo: &str, file: &str) {
    let sum_file = format!("{}.{}", file, algo);
    let algo_upper = algo.to_uppercase();

    // Проверка существования файлов
    if !Path::new(file).exists() {
        eprintln!("{}Ошибка: Файл '{}' не найден.{}", RED, file, NC);
        return;
    }
    if !Path::new(&sum_file).exists() {
        eprintln!("{}Ошибка: Нет хеш-файла '{}'.{}", RED, sum_file, NC);
        return;
    }

    let content = fs::read_to_string(&sum_file).expect("Не удалось прочитать хеш-файл");
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
        println!("  [{RED}FAIL{NC}] [{algo_upper}] {file} (Ошибка хеша!)");
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
        _ => "неподдерживаемый".to_string(),
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
    let mut files_found = false;
    
    for path in paths.filter_map(|e| e.ok()) {
        let name = path.file_name().into_string().unwrap();
        
        // Перевіряємо, чи є файл контрольною сумою
        if name.ends_with(".md5") || name.ends_with(".sha1") || name.ends_with(".sha256") || name.ends_with(".sha512") {
            files_found = true;
            let parts: Vec<&str> = name.split('.').collect();
            let algo = parts.last().unwrap();
            let base = parts[..parts.len() - 1].join(".");
            
            // Додаткова перевірка: чи починається файл з "floren"
            if base.to_lowercase().starts_with("floren") {
                check_hash(algo, &base);
            }
        }
    }

    if !files_found {
        eprintln!(
            "{}Файлов нет или файлы для другого скрипта. Чтобы запустить команду, скачайте скрипты с GitHub: https://github.com/Floren-Team{}",
            RED, NC
        );
    }
}
