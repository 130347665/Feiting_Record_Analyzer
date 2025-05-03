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
    fn find_all_consecutive_patterns(&self, pattern_input: &str) -> Vec<Vec<&DrawResult>> {
        let trimmed_input = pattern_input.trim();
        if self.draws.is_empty() || trimmed_input.is_empty() {
            return vec![];
        }

        // 分割不同位置的模式，這次 **保留** 空字串，但去除每個部分的前後空白
        let position_patterns: Vec<&str> = trimmed_input
            .split(',')
            .map(|s| s.trim()) // 去除每個部分的前後空白
            .collect();

        // 如果分割後所有部分都是空的（例如輸入只有逗號），則返回空
        if position_patterns.iter().all(|s| s.is_empty()) {
             return vec![];
        }

        // 按期號升序排序
        let mut draws_sorted: Vec<&DrawResult> = self.draws.iter().collect();
        draws_sorted.sort_by(|a, b| a.drawNumber.cmp(&b.drawNumber));

        // --- 確定基準模式長度 (pattern_len) ---
        // 找到第一個非空模式來確定長度
        let mut pattern_len = 0;
        let mut first_valid_pattern_parts: Option<Vec<&str>> = None;

        for pattern_str in &position_patterns {
            if !pattern_str.is_empty() {
                let parts: Vec<&str> = pattern_str
                    .as_bytes()
                    .chunks(3) // 假設中文佔3字節
                    .filter_map(|c| std::str::from_utf8(c).ok())
                    .filter(|s| !s.trim().is_empty()) // 確保解析出的部分不是空的
                    .collect();

                if !parts.is_empty() {
                    pattern_len = parts.len();
                    first_valid_pattern_parts = Some(parts);
                    break; // 找到第一個有效的就停止
                }
            }
        }

        // 如果找不到任何有效的非空模式，或者第一個有效模式解析不出長度，則無法匹配
        if pattern_len == 0 || first_valid_pattern_parts.is_none() {
            return vec![];
        }
        // --- 基準模式長度確定完畢 ---


        // 儲存所有匹配的結果
        let mut all_matches: Vec<Vec<&DrawResult>> = vec![];

        // 檢查每個可能的起始位置
        for start_idx in 0..=draws_sorted.len().saturating_sub(pattern_len) {
            let mut matches_all_required_positions = true; // 標記是否匹配了所有 **需要** 檢查的位置

            // 遍歷定義的模式（包括空字串代表的跳過位置）
            'position_loop: for (position_idx, position_pattern) in position_patterns.iter().enumerate() {

                // ******** 核心修改：檢查是否需要跳過此位置 ********
                if position_pattern.is_empty() {
                    // 如果當前模式是空的，代表用戶想跳過這個位置的檢查
                    continue 'position_loop; // 直接跳到下一個 position_pattern
                }
                // ******** 跳過檢查邏輯結束 ********


                // --- 如果不需要跳過，則執行檢查 ---
                let position_to_check = position_idx + 1; // 位置從1開始計數

                // 解析當前位置的模式 (只有非空時才解析)
                let pattern_parts: Vec<&str> = position_pattern
                    .as_bytes()
                    .chunks(3)
                    .filter_map(|c| std::str::from_utf8(c).ok())
                    .filter(|s| !s.trim().is_empty())
                    .collect();

                // **健壯性檢查**: 確保解析出的模式長度與基準長度一致
                if pattern_parts.len() != pattern_len {
                    // 如果這個非空模式的長度與基準長度不同，則認為格式錯誤，匹配失敗
                    matches_all_required_positions = false;
                    break 'position_loop;
                }


                // 檢查這個起始位置開始的 pattern_len 個期數是否匹配當前位置的模式
                for i in 0..pattern_len {
                    // 索引 i 對於 pattern_parts 是安全的，因為上面檢查了長度一致性
                    let draw = draws_sorted[start_idx + i];

                    let numbers: Vec<u32> = draw.result
                        .split(',')
                        .filter_map(|s| s.trim().parse::<u32>().ok())
                        .collect();

                    // 確保開獎結果有足夠的數字來檢查這個位置
                    if position_to_check > numbers.len() {
                        matches_all_required_positions = false;
                        break 'position_loop;
                    }

                    let num_at_position = numbers[position_to_check - 1]; // 索引從0開始

                    let is_match = match pattern_parts[i] {
                        "單" | "单" => num_at_position % 2 != 0,
                        "雙" | "双" => num_at_position % 2 == 0,
                        "大" => num_at_position >= 6,
                        "小" => num_at_position < 6,
                        _ => false,
                    };

                    if !is_match {
                        // 只要有一個不匹配，當前 start_idx 的嘗試就失敗了
                        matches_all_required_positions = false;
                        break 'position_loop; // 跳出對所有位置的檢查，處理下一個 start_idx
                    }
                } // end inner loop for i
            } // end 'position_loop (遍歷所有定義的模式)

            // 如果成功匹配了所有 **需要檢查** 的位置
            if matches_all_required_positions {
                all_matches.push(draws_sorted[start_idx..start_idx + pattern_len].to_vec());
            }
        } // end outer loop for start_idx

        all_matches
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
                ui.label("🔍 連續期數模式:");
                ui.text_edit_singleline(&mut self.filter);
            });
            
            // 添加模式格式說明
            ui.label("💡 格式說明: 單單雙,雙雙單 表示第1位匹配「單單雙」，第2位匹配「雙雙單」");
            ui.label("💡 可用模式：單/雙/大/小 (大表示≥6，小表示<6)");
    
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
                                // 獲取所有數字，用於顯示指定位置的數字
                                let numbers: Vec<String> = draw.result
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();
                                    
                                // 顯示結果，包括指定位置的數字
                                let position_patterns: Vec<&str> = self.filter.split(',').collect();
                                let mut position_info = String::new();
                                
                                for (idx, _) in position_patterns.iter().enumerate() {
                                    let position = idx + 1;
                                    if position <= numbers.len() {
                                        let num = &numbers[position - 1];
                                        position_info.push_str(&format!("第{}位: {}, ", position, num));
                                    }
                                }
                                
                                // 去掉最後的逗號和空格
                                if !position_info.is_empty() {
                                    position_info = position_info[..position_info.len() - 2].to_string();
                                }
                                
                                let beijing_time = Self::timestamp_to_beijing_time(draw.drawTime);
                                ui.label(format!("✅ [{}] {} → {} ({})", beijing_time, draw.drawNumber, draw.result, position_info));
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
                        let numbers: Vec<String> = draw.result
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                            
                        let first_few_nums = if numbers.len() >= 3 {
                            format!("前三號：{}, {}, {}", numbers[0], numbers[1], numbers[2])
                        } else if !numbers.is_empty() {
                            format!("首號：{}", numbers[0])
                        } else {
                            "無數據".to_string()
                        };
                        
                        ui.label(format!("📊 {} → {} ({})", draw.drawNumber, draw.result, first_few_nums));
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