use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use serde::Serialize;


#[derive(Serialize)]
#[serde(untagged)]
enum ConfigValue {
    Single(String),
    Nested(HashMap<String, String>),
}


fn main() -> io::Result<()> {
    // ファイルを指定
    let path = "config.txt";  // 任意の設定ファイルのパスを指定

    // ファイルを開く
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    // 設定を格納するHashMap
    let mut config: HashMap<String, ConfigValue> = HashMap::new();
    let mut current_section: Option<HashMap<String, String>> = None;

    // 各行を処理
    for line in reader.lines() {
        let line = line?;

        // コメント行や空行をスキップ
        if line.trim().is_empty() || line.trim().starts_with("#") {
            continue;
        }

        // セクション（例: logのようなグループ）を処理
        if line.contains(".") {
            // "log.file" のような形式を処理
            let mut parts = line.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    let key = key.trim();
                    let value = value.trim();

                    // セクションを作成または更新
                    let sections: Vec<&str> = key.split('.').collect();
                    let section_name = sections[0];

                    // セクションがすでに存在していれば更新、なければ新しく作成
                    let section = config.entry(section_name.to_string()).or_insert_with(|| ConfigValue::Nested(HashMap::new()));

                    // Nestedの場合のみ処理
                    if let ConfigValue::Nested(ref mut section_map) = section {
                        section_map.insert(sections[1].to_string(), value.to_string());
                    }                }
            }
        } else if let Some((key, value)) = line.split_once('=') {
            // "."が含まれない単純なキーと値のペアを処理
            let key = key.trim();
            let value = value.trim();

            config.insert(key.to_string(), ConfigValue::Single(value.to_string()));
        }
    }

    // 結果の表示 (JSON形式)
    let json_output = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", json_output);

    Ok(())
}
