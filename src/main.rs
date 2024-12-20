use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use serde::Serialize;
use std::process;

#[derive(Serialize)]
#[serde(untagged)]
enum ConfigValue {
    Single(String), // セクション（例: log.file.dir のような形式）に重複部分が無い時
    Nested(HashMap<String, ConfigValue>), // セクション（例: log.file.dir のような形式）に重複部分が有る時
}

fn main() {
    // スキーマファイルを指定
    let folder: &str = "input_check/";
    let file_path: &str = "schema.txt";
    let path: String = [folder, file_path].concat();

    // スキーマファイルを読み込む
    let schema: HashMap<String, String> = match read_config_schema(&path) {
        Ok(schema) => schema,
        Err(err) => {
            eprintln!("Error reading schema file: {}", err);
            process::exit(1);
        }
    };

    // コマンドライン引数を取得
    let args: Vec<String> = env::args().collect();

    // 引数が渡されているか確認
    if args.len() < 2 {
        eprintln!("Error: Missing file name argument.\nUsage: {} <file_name>", args[0]);
        process::exit(1); // 終了コード 1 でプログラムを終了
    }

    // ファイル名を取得
    let file_name = &args[1];

    // ファイルを開いて内容を読み込む
    if let Err(e) = read_and_print_file(file_name, &schema) {
        eprintln!("Error: Failed to read file '{}': {}", file_name, e);
        process::exit(1); // エラー時に終了コード 1 で終了
    }
}

// スキーマファイルを読み込んでHashMapで返す。
fn read_config_schema(path: &str) -> io::Result<HashMap<String, String>> {
    let mut schema: HashMap<String, String> = HashMap::new();

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() || line.trim().starts_with('#') || line.trim().starts_with(';'){
            continue; // 空行やコメント行をスキップ
        }

        // 各行を -> で分割し、キーと型を抽出
        if let Some((key, value_type)) = line.split_once("->") {
            let key = key.trim().to_string();
            let value_type = value_type.trim().to_string();
            schema.insert(key, value_type);
        } else {
            eprintln!("Warning: Invalid line format: '{}'", line);
        }
    }

    Ok(schema)
}

// confファイル読み込み
fn read_and_print_file(file_name: &str, schema: &HashMap<String, String>) -> io::Result<()> {
    let folder = "assets/"; // フォルダ
    // ファイルを指定
    let path = [folder, file_name].concat();

    // ファイルを開く
    let file: File = File::open(path)?;
    let reader: io::BufReader<File> = io::BufReader::new(file);

    // 設定を格納するHashMap
    let mut config: HashMap<String, ConfigValue> = HashMap::new();

    // 各行を処理
    for line_result in reader.lines() {
        let line: String = line_result?;

        // コメント行や空行をスキップ
        if line.trim().is_empty() || line.trim().starts_with('#') || line.trim().starts_with(';'){
            continue;
        }

        // セクションを処理
        if line.contains('.') {
            let mut parts: std::str::SplitN<'_, char> = line.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    let key: &str = key.trim();
                    let value: &str = value.trim();

                    // "."で区切られたセクションを処理
                    let sections: Vec<&str> = key.split('.').collect();
                    let mut current_map: &mut HashMap<String, ConfigValue> = &mut config;

                    // 最後のセクションを除いて、ネストされたマップを作成または取得
                    for section in &sections[..sections.len() - 1] {
                        let entry: &mut ConfigValue = current_map
                            .entry(section.to_string())
                            .or_insert_with(|| ConfigValue::Nested(HashMap::new()));

                        // 値が Nested の場合にのみ更新
                        if let ConfigValue::Nested(ref mut section_map) = entry {
                            current_map = section_map;
                        } else {
                            panic!(
                                "Unexpected non-nested value for section '{}'",
                                section
                            );
                        }
                    }

                    // スキーマファイルのキーが存在するか確認
                    if schema.contains_key(key) {
                        let schema_value = schema.get(key)
                            .map(|v| v.clone())  // Option<&String>をStringに変換
                            .unwrap_or_else(|| String::from("default_value")); // 存在しない場合はデフォルト値

                        // スキーマファイル通りの入力値かどうかチェック
                        validate_type(&key, &value.to_string(), &schema_value);
                    }

                    // 最後のセクションを設定
                    current_map.insert(
                        sections.last().unwrap().to_string(),
                        ConfigValue::Single(value.to_string()),
                    );
                }
            }
        } else if let Some((key, value)) = line.split_once('=') {
            // "."が含まれない単純なキーと値のペアを処理
            let key: &str = key.trim();
            let value: &str = value.trim();

            // スキーマファイルのキーが存在するか確認
            if schema.contains_key(key) {
                let schema_value = schema.get(&key.to_string())
                    .map(|v| v.clone())  // Option<&String>をStringに変換
                    .unwrap_or_else(|| String::from("default_value")); // 存在しない場合はデフォルト値

                // スキーマファイル通りの入力値かどうかチェック
                validate_type(&key, &value.to_string(), &schema_value);
            }

            config.insert(key.to_string(), ConfigValue::Single(value.to_string()));
        }
    }

    // 結果の表示 (JSON形式)
    let json_output: String = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", json_output);

    Ok(())
}

// スキーマファイルのルールに則っているかチェック
fn validate_type(key: &str, value: &str, schema_value: &str) {
    match schema_value {
        "bool" => {
            // 値がbool型であることを確認
            let parsed_value: Result<bool, _> = value.trim().to_lowercase().parse();

            if parsed_value.is_err() {
                eprintln!("スキーマチェックエラー： {}は{}の形式ではありません。", key, schema_value);
                std::process::exit(1); // エラーメッセージだけを表示して終了
            }
        },
        "integer" => {
            // 値が数値型(i32)であることを確認
            let parsed_value: Result<i32, _> = value.trim().parse();
            
            if parsed_value.is_err() {
                eprintln!("スキーマチェックエラー： {}は{}の形式ではありません。", key, schema_value);
                std::process::exit(1);
            }
        },
        "String" => {
            // `value`はすでに`&str`型なのでここでは何もしない。
        },
        _ => {},
    }
}