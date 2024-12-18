use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
enum ConfigValue {
    Single(String), // セクション（例: log.file.dir のような形式）に重複部分が無い時
    Nested(HashMap<String, ConfigValue>), // セクション（例: log.file.dir のような形式）に重複部分が有る時
}

fn main() -> io::Result<()> {
    // ファイルを指定
    let path: &str = "sample.conf"; // 任意の設定ファイルのパスを指定

    // ファイルを開く
    let file: File = File::open(path)?;
    let reader: io::BufReader<File> = io::BufReader::new(file);

    // 設定を格納するHashMap
    let mut config: HashMap<String, ConfigValue> = HashMap::new();

    // 各行を処理
    for line_result in reader.lines() {
        let line: String = line_result?;

        // コメント行や空行をスキップ
        if line.trim().is_empty() || line.trim().starts_with('#') {
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

            config.insert(key.to_string(), ConfigValue::Single(value.to_string()));
        }
    }

    // 結果の表示 (JSON形式)
    let json_output: String = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", json_output);

    Ok(())
}
