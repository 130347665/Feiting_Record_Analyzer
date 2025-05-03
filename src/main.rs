#![windows_subsystem = "windows"]  // 添加這一行到 main.rs 文件頂部
use std::collections::HashMap;

use eframe::{egui, App, Frame};
use reqwest::Client;
use serde::Deserialize;
use tokio::runtime::Runtime;
use chrono::{Utc, FixedOffset};
use chrono::DateTime;
use chrono::TimeZone;
#[derive(Deserialize, Debug, Clone)]
struct DrawResult {
    drawNumber: String,
    drawTime: u64,
    result: String,
    detail: String,
}
// 定義遊戲類型
enum GameType {
    SGFeiTing,      // SG飛艇
    XingYunFeiTing, // 幸運飛艇
    JiSuFeiTing,    // 急速飛艇
    JiSuSaiChe,     // 急速賽車
}

impl GameType {
    fn to_url(&self) -> &str {
        match self {
            GameType::SGFeiTing => "SGFT",
            GameType::XingYunFeiTing => "XYFT",
            GameType::JiSuFeiTing => "LUCKYSB",
            GameType::JiSuSaiChe => "PK10JSC",
        }
    }
    
    fn to_name(&self) -> &str {
        match self {
            GameType::SGFeiTing => "SG飛艇",
            GameType::XingYunFeiTing => "幸運飛艇",
            GameType::JiSuFeiTing => "急速飛艇",
            GameType::JiSuSaiChe => "急速賽車",
        }
    }
}
pub struct MyApp {
    draws: Vec<DrawResult>,
    filter: String,
    status: String,
    rt: Runtime,
    cookie: String, // 新增：用於存儲用戶輸入的 Cookie
    current_game: GameType, // 當前選擇的遊戲類型
    game_checkboxes: HashMap<String, bool>, // 存儲遊戲選擇狀態的HashMap
}


impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use egui::{FontData, FontDefinitions, FontFamily};
    
        // ✅ 修改字型配置
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            std::sync::Arc::new(FontData::from_static(include_bytes!("C:/Windows/Fonts/msyh.ttc"))), // 微軟正黑體
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());
    
        cc.egui_ctx.set_fonts(fonts);
        // 初始化遊戲選擇狀態
        let mut game_checkboxes = HashMap::new();
        game_checkboxes.insert("SG飛艇".to_string(), true); // 默認選擇SG飛艇
        game_checkboxes.insert("幸運飛艇".to_string(), false);
        game_checkboxes.insert("急速飛艇".to_string(), false);
        game_checkboxes.insert("急速賽車".to_string(), false);
    
        Self {
            draws: vec![],
            filter: "".into(),
            status: "請點擊『獲取數據』".into(),
            rt: tokio::runtime::Runtime::new().unwrap(),
            cookie: String::new(), // 新增：用於存儲用戶輸入的 Cookie
            current_game: GameType::SGFeiTing, // 默認遊戲類型
            game_checkboxes,
        }
    }
    
    fn fetch_data(&mut self) {
        // 檢查Cookie是否已輸入
        if self.cookie.trim().is_empty() {
            self.status = "❌ 請先輸入Cookie後再獲取數據".into();
            return;
        }

        // 獲取當前選中的遊戲類型
        let game_url = self.current_game.to_url();
        let game_name = self.current_game.to_name();
        self.status = "📡 資料抓取中...".into();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("accept", "application/json, text/plain, */*".parse().unwrap());
        headers.insert("accept-language", "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7,lb;q=0.6,zh-CN;q=0.5".parse().unwrap());
        headers.insert("priority", "u=1, i".parse().unwrap());
        headers.insert("referer", "https://www.15678.vip/player/lottery/LUCKYSB".parse().unwrap());
        headers.insert("sec-ch-ua", "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\"".parse().unwrap());
        headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
        headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
        headers.insert("sec-fetch-dest", "empty".parse().unwrap());
        headers.insert("sec-fetch-mode", "cors".parse().unwrap());
        headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36".parse().unwrap());
        headers.insert(reqwest::header::COOKIE, self.cookie.parse().unwrap());

        let client = Client::new();
        let result = self.rt.block_on(async {
            client
                .get(format!("https://www.15678.vip/member/dayResult?lottery={}", game_url))
                .headers(headers)
                .send()
                .await?
                .json::<Vec<DrawResult>>()
                .await
        });

        match result {
            Ok(data) => {
                self.draws = data;
                self.status = format!("✅ 共載入 {} 筆資料", self.draws.len());
            }
            Err(err) => {
                self.status = format!("❌ 載入失敗: {}", err);
            }
        }
    }
    // 修改後的函數會返回所有匹配的連續模式
    fn find_all_consecutive_patterns(&self, pattern: &str) -> Vec<Vec<&DrawResult>> {
        if self.draws.is_empty() || pattern.is_empty() {
            return vec![];
        }
        
        // 解析模式
        let pattern_parts: Vec<&str> = pattern
            .as_bytes()
            .chunks(3)
            .filter_map(|c| std::str::from_utf8(c).ok())
            .collect();
            
        if pattern_parts.is_empty() {
            return vec![];
        }
        
        // 按期號升序排序的臨時draws集合
        let mut draws_sorted: Vec<&DrawResult> = self.draws.iter().collect();
        draws_sorted.sort_by(|a, b| a.drawNumber.cmp(&b.drawNumber));
        
        // 儲存所有匹配的結果
        let mut all_matches: Vec<Vec<&DrawResult>> = vec![];
        
        // 查找連續匹配模式的期數
        let pattern_len = pattern_parts.len();
        
        // 檢查每個可能的起始位置
        for start_idx in 0..=draws_sorted.len().saturating_sub(pattern_len) {
            let mut matches = true;
            
            // 檢查這個起始位置開始的pattern_len個期數是否匹配模式
            for i in 0..pattern_len {
                let draw = draws_sorted[start_idx + i];
                // 取得這一期的第一個數字
                if let Some(first_num_str) = draw.result.split(',').next() {
                    if let Ok(first_num) = first_num_str.trim().parse::<u32>() {
                        let is_match = match pattern_parts[i] {
                            "單" | "单" => first_num % 2 != 0,
                            "雙" | "双" => first_num % 2 == 0,
                            "大" => first_num >= 6,
                            "小" => first_num < 6,
                            _ => false,
                        };
                        
                        if !is_match {
                            matches = false;
                            break;
                        }
                    } else {
                        matches = false;
                        break;
                    }
                } else {
                    matches = false;
                    break;
                }
            }
            
            // 如果找到完整匹配，加入結果集
            if matches {
                all_matches.push(draws_sorted[start_idx..start_idx + pattern_len].to_vec());
                // 可以選擇跳過已匹配的期數，以避免重疊
                // start_idx += pattern_len - 1;
            }
        }
        
        all_matches
    }
    // 新增：查找連續期數匹配模式的函數
    fn find_consecutive_patterns(&self, pattern: &str) -> Vec<&DrawResult> {
        if self.draws.is_empty() || pattern.is_empty() {
            return vec![];
        }
        
        // 解析模式
        let pattern_parts: Vec<&str> = pattern
            .as_bytes()
            .chunks(3)
            .filter_map(|c| std::str::from_utf8(c).ok())
            .collect();
            
        if pattern_parts.is_empty() {
            return vec![];
        }
        
        // 按期號升序排序的臨時draws集合
        let mut draws_sorted: Vec<&DrawResult> = self.draws.iter().collect();
        draws_sorted.sort_by(|a, b| a.drawNumber.cmp(&b.drawNumber));
        
        // 查找連續匹配模式的期數
        let pattern_len = pattern_parts.len();
        
        // 檢查每個可能的起始位置
        for start_idx in 0..=draws_sorted.len().saturating_sub(pattern_len) {
            let mut matches = true;
            
            // 檢查這個起始位置開始的pattern_len個期數是否匹配模式
            for i in 0..pattern_len {
                let draw = draws_sorted[start_idx + i];
                // 取得這一期的第一個數字
                if let Some(first_num_str) = draw.result.split(',').next() {
                    if let Ok(first_num) = first_num_str.trim().parse::<u32>() {
                        let is_match = match pattern_parts[i] {
                            "單" | "单" => first_num % 2 != 0,
                            "雙" | "双" => first_num % 2 == 0,
                            "大" => first_num >= 6,
                            "小" => first_num < 6,
                            _ => false,
                        };
                        
                        if !is_match {
                            matches = false;
                            break;
                        }
                    } else {
                        matches = false;
                        break;
                    }
                } else {
                    matches = false;
                    break;
                }
            }
            
            // 如果找到完整匹配
            if matches {
                return draws_sorted[start_idx..start_idx + pattern_len].to_vec();
            }
        }
        
        // 沒找到完整匹配
        vec![]
    }
    // 新增：時間戳轉北京時間函數（支持毫秒級時間戳）
    fn timestamp_to_beijing_time(timestamp: u64) -> String {
        // 檢查是否為毫秒級時間戳（13位數字）
        let timestamp_seconds = if timestamp > 10000000000 {
            // 毫秒轉秒
            timestamp / 1000
        } else {
            // 已經是秒級
            timestamp
        };
        
        // 獲取毫秒部分用於顯示
        let milliseconds = if timestamp > 10000000000 {
            timestamp % 1000
        } else {
            0
        };
        
        // 創建一個UTC時間
        let utc_time = match Utc.timestamp_opt(timestamp_seconds as i64, 0) {
            chrono::offset::LocalResult::Single(dt) => dt,
            _ => return "時間格式錯誤".to_string(),
        };
        
        // 創建北京時區 (UTC+8)
        let beijing_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
        
        // 轉換為北京時間
        let beijing_time: DateTime<FixedOffset> = utc_time.with_timezone(&beijing_timezone);
        
        // 格式化時間（包含毫秒）
        if milliseconds > 0 {
            beijing_time.format("%Y-%m-%d %H:%M:%S").to_string() + &format!(".{:03}", milliseconds)
        } else {
            beijing_time.format("%Y-%m-%d %H:%M:%S").to_string()
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🏁 飛艇歷史數據查詢器");
            // 新增：Cookie輸入框
            ui.horizontal(|ui| {
                ui.label("🔑 Cookie:");
                ui.text_edit_singleline(&mut self.cookie);
            });

            // 遊戲類型選擇區域
            ui.heading("選擇遊戲類型");
            
            // 使用單選按鈕來選擇不同遊戲類型
            ui.horizontal(|ui| {
                if ui.radio(*self.game_checkboxes.get("SG飛艇").unwrap_or(&false), "SG飛艇").clicked() {
                    // 更新所有遊戲選擇狀態
                    for (game, checked) in self.game_checkboxes.iter_mut() {
                        *checked = game == "SG飛艇";
                    }
                    self.current_game = GameType::SGFeiTing;
                    self.draws.clear(); // 清除之前的數據
                }
                
                if ui.radio(*self.game_checkboxes.get("幸運飛艇").unwrap_or(&false), "幸運飛艇").clicked() {
                    // 更新所有遊戲選擇狀態
                    for (game, checked) in self.game_checkboxes.iter_mut() {
                        *checked = game == "幸運飛艇";
                    }
                    self.current_game = GameType::XingYunFeiTing;
                    self.draws.clear(); // 清除之前的數據
                }
                
                if ui.radio(*self.game_checkboxes.get("急速飛艇").unwrap_or(&false), "急速飛艇").clicked() {
                    // 更新所有遊戲選擇狀態
                    for (game, checked) in self.game_checkboxes.iter_mut() {
                        *checked = game == "急速飛艇";
                    }
                    self.current_game = GameType::JiSuFeiTing;
                    self.draws.clear(); // 清除之前的數據
                }
                
                if ui.radio(*self.game_checkboxes.get("急速賽車").unwrap_or(&false), "急速賽車").clicked() {
                    // 更新所有遊戲選擇狀態
                    for (game, checked) in self.game_checkboxes.iter_mut() {
                        *checked = game == "急速賽車";
                    }
                    self.current_game = GameType::JiSuSaiChe;
                    self.draws.clear(); // 清除之前的數據
                }
            });

            if ui.button("📥 獲取數據").clicked() {
                self.fetch_data();
            }

            ui.horizontal(|ui| {
                ui.label("🔍 連續期數模式（例如：單單雙）:");
                ui.text_edit_singleline(&mut self.filter);
            });

            ui.label(&self.status);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if !self.filter.is_empty() && !self.draws.is_empty() {
                    // 查找所有連續期數匹配模式
                    let all_matched = self.find_all_consecutive_patterns(&self.filter);
                    
                    if !all_matched.is_empty() {
                        ui.heading(format!("🎯 找到 {} 組連續模式：", all_matched.len()));
                        
                        for (group_idx, matched_group) in all_matched.iter().enumerate() {
                            // 默認展開所有組
                            ui.heading(format!("🔍 匹配組 #{} (期數: {})", group_idx + 1, matched_group.len()));
                            
                            // 直接顯示每個組的結果，不使用折疊面板
                            for draw in matched_group {
                                let first_num = draw.result.split(',').next().unwrap_or("?");
                                let beijing_time = Self::timestamp_to_beijing_time(draw.drawTime);
                                ui.label(format!("✅ [{}] {} → {}（首號：{}）", beijing_time, draw.drawNumber, draw.result, first_num));
                            }
                            
                            // 添加分隔線以區分不同組
                            if group_idx < all_matched.len() - 1 {
                                ui.separator();
                            }
                        }
                    } else {
                        ui.label("❌ 沒有找到符合連續模式的結果");
                    }
                } else if !self.draws.is_empty() {
                    // 顯示所有資料
                    ui.heading("📋 全部數據：");
                    let mut sorted_draws = self.draws.clone();
                    sorted_draws.sort_by(|a, b| a.drawNumber.cmp(&b.drawNumber));
                    
                    for draw in sorted_draws {
                        let first_num = draw.result.split(',').next().unwrap_or("?");
                        ui.label(format!("📊 {} → {}（首號：{}）", draw.drawNumber, draw.result, first_num));
                    }
                }
            });
        });
    }
}

// 原始的匹配函數保留，但不再使用
fn matches_rule(result: &str, rule_input: &str) -> bool {
    let nums: Vec<u32> = result
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let rule_parts: Vec<&str> = rule_input
        .as_bytes()
        .chunks(3)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect();

    if nums.len() < rule_parts.len() {
        return false;
    }

    // 倒序逐一匹配
    for i in 0..rule_parts.len() {
        let n = nums[nums.len() - 1 - i];
        let rule = rule_parts[rule_parts.len() - 1 - i];

        let is_match = match rule {
            "單" | "单" => n % 2 != 0,
            "雙" | "双" => n % 2 == 0,
            "大" => n >= 6,
            "小" => n < 6,
            _ => false,
        };

        if !is_match {
            return false;
        }
    }

    true
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "飛艇記錄查詢器",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}