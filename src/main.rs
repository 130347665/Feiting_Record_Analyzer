#![windows_subsystem = "windows"]  // 添加這一行到 main.rs 文件頂部
use std::collections::{HashMap, HashSet};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameType {
    SGFeiTing,       // SG飛艇
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
    fn all() -> impl Iterator<Item = Self> {
        [
            GameType::SGFeiTing,
            GameType::XingYunFeiTing,
            GameType::JiSuFeiTing,
            GameType::JiSuSaiChe,
        ]
        .iter()
        .copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RulePart {
    positions: Vec<usize>, // Positions to check (1-based index)
    pattern: Vec<char>,    // Pattern characters (e.g., ['单', '单', '单', '单'])
}

#[derive(Debug, PartialEq)]
pub enum FindPatternError {
    InvalidInputFormat(String),
    InvalidPosition(String),
    EmptyPattern,
    DrawDataError(String), // Errors related to accessing/parsing draw data (optional)
}


pub struct MyApp {
    draws: Vec<DrawResult>,
    filter: String,
    status: String,
    rt: Runtime,
    cookie: String, // 新增：用於存儲用戶輸入的 Cookie
    current_game: GameType, // 當前選擇的遊戲類型
}

fn determine_characteristic(num: u32) -> String {
    let size = if num >= 6 { '大' } else { '小' };
    // Use consistent Traditional Chinese characters for display
    let parity = if num % 2 == 0 { '雙' } else { '單' };
    format!("{}{}", size, parity)
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use egui::{FontData, FontDefinitions, FontFamily};
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
             // Consider embedding or using a relative path
            std::sync::Arc::new(FontData::from_static(include_bytes!(
                "C:/Windows/Fonts/msyh.ttc" // WARNING: Hardcoded path
            ))),
        );
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "my_font".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().push("my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            draws: vec![],
            filter: "".into(),
            status: "請選擇遊戲並輸入Cookie後點擊『獲取數據』".into(),
            rt: tokio::runtime::Runtime::new().unwrap(),
            cookie: String::new(),
            current_game: GameType::SGFeiTing,
        }
    }

    fn fetch_data(&mut self) {
        if self.cookie.trim().is_empty() {
            self.status = "❌ 請先輸入Cookie後再獲取數據".into();
            return;
        }
        let game_url = self.current_game.to_url();
        let game_name = self.current_game.to_name();
        self.status = format!("📡 ({}) 資料抓取中...", game_name).into();
        let cookie = self.cookie.clone();
        let current_game_url = self.current_game.to_url().to_string();

        let result = self.rt.block_on(async {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("accept", "application/json, text/plain, */*".parse().unwrap());
            headers.insert("accept-language", "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7".parse().unwrap());
            headers.insert("sec-ch-ua", "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\"".parse().unwrap());
            headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
            headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
            headers.insert("sec-fetch-dest", "empty".parse().unwrap());
            headers.insert("sec-fetch-mode", "cors".parse().unwrap());
            headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
            headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36".parse().unwrap());
            headers.insert("referer", format!("https://www.15678.vip/player/lottery/{}", current_game_url).parse().unwrap());
             if let Ok(cookie_header) = cookie.parse() { headers.insert(reqwest::header::COOKIE, cookie_header); }
             else { eprintln!("Warning: Failed to parse cookie header"); }

            let client = Client::builder().build().map_err(|e| e.to_string())?;
            client.get(format!("https://www.15678.vip/member/dayResult?lottery={}", current_game_url))
                .headers(headers).send().await.map_err(|e| e.to_string())?
                .json::<Vec<DrawResult>>().await.map_err(|e| e.to_string())
        });

        match result {
            Ok(data) => {
                self.draws = data;
                 self.draws.sort_by(|a, b| a.drawNumber.cmp(&b.drawNumber));
                self.status = format!("✅ ({}) 共載入 {} 筆資料", game_name, self.draws.len());
            }
            Err(err) => {
                self.status = format!("❌ ({}) 載入失敗: {}", game_name, err);
                self.draws.clear();
            }
        }
    }

    fn parse_rule_string(&self, pattern_input: &str) -> Result<Vec<RulePart>, FindPatternError> {
        let mut rules = Vec::new();
        let mut chars = pattern_input.chars().peekable();
        while chars.peek().is_some() {
            while let Some(&c) = chars.peek() { if c.is_whitespace() { chars.next(); } else { break; } }
            if chars.peek().is_none() { break };
            let mut positions_str = String::new();
            while let Some(&c) = chars.peek() { if c.is_ascii_digit() { positions_str.push(chars.next().unwrap()); } else { break; } }
            if positions_str.is_empty() { return Err(FindPatternError::InvalidInputFormat(format!("規則必須以位置數字開頭，但找到 '{:?}'", chars.peek()))); }
            let positions: Vec<usize> = positions_str.chars().map(|c| c.to_digit(10).unwrap() as usize).collect();
             if positions.iter().any(|&p| p == 0) || positions.len() != positions_str.len() { return Err(FindPatternError::InvalidPosition(format!("無效的位置數字 '{}'", positions_str))); }
            let mut pattern_chars = Vec::new();
            while let Some(&c) = chars.peek() {
                 if !c.is_ascii_digit() && !c.is_whitespace() {
                    match c { '单' | '單' | '双' | '雙' | '大' | '小' => pattern_chars.push(chars.next().unwrap()),
                         _ => return Err(FindPatternError::InvalidInputFormat(format!("無效的模式字符 '{}'", c))), }
                 } else { break; }
            }
            if pattern_chars.is_empty() { return Err(FindPatternError::EmptyPattern); }
            rules.push(RulePart { positions, pattern: pattern_chars });
        }
         if rules.is_empty() && !pattern_input.trim().is_empty() { return Err(FindPatternError::InvalidInputFormat("輸入解析後未找到任何有效規則".to_string())); }
        Ok(rules)
    }

    fn check_num_match(&self, num: u32, pattern_char: char) -> bool {
        match pattern_char {
            '单' | '單' => num % 2 != 0, '双' | '雙' => num % 2 == 0,
            '大' => num >= 6, '小' => num < 6, _ => false,
        }
    }

    // --- Modified find_matching_sequences to return parsed rules ---
    fn find_matching_sequences(&self) -> Result<(Vec<Vec<&DrawResult>>, Vec<RulePart>), FindPatternError>
    {
        let trimmed_input = self.filter.trim(); // Use self.filter directly
        if self.draws.is_empty() || trimmed_input.is_empty() {
            // Return empty rules if no input, successful parse but no rules essentially
             return Ok((vec![], vec![]));
        }

        // 1. Parse the input string first
        let rules = self.parse_rule_string(trimmed_input)?; // Propagate parse error
        if rules.is_empty() {
             return Ok((vec![], rules)); // Parsed ok, but no rules found
        }

        // 2. Prepare data (already sorted)
        let draws_sorted = &self.draws;

        // 3. Find matches
        let mut matched_sequence_indices: HashSet<(usize, usize)> = HashSet::new();
        for rule_part in &rules { // Use the parsed rules
            let pattern_len = rule_part.pattern.len();
            if pattern_len == 0 || draws_sorted.len() < pattern_len { continue; }

            'window_loop: for start_idx in 0..=draws_sorted.len() - pattern_len {
                let window = &draws_sorted[start_idx..start_idx + pattern_len];
                let mut window_matches_this_rule = false;
                for &pos in &rule_part.positions {
                    if pos == 0 { continue; }
                    let mut current_pos_matches_all_draws = true;
                    for i in 0..pattern_len {
                        let draw = &window[i];
                        let target_char = rule_part.pattern[i];
                        let numbers_res: Result<Vec<u32>, _> = draw.result.split(',').map(|s| s.trim().parse::<u32>()).collect();
                        match numbers_res {
                             Ok(numbers) => {
                                if pos > numbers.len() { current_pos_matches_all_draws = false; break; }
                                let num_at_pos = numbers[pos - 1];
                                if !self.check_num_match(num_at_pos, target_char) { current_pos_matches_all_draws = false; break; }
                            }
                            Err(_) => { current_pos_matches_all_draws = false; break; } // Handle parse error for draw result
                        }
                    }
                    if current_pos_matches_all_draws { window_matches_this_rule = true; break; }
                }
                if window_matches_this_rule { matched_sequence_indices.insert((start_idx, pattern_len)); }
            }
        }

        // 4. Convert indices to results
        let mut sorted_indices: Vec<(usize, usize)> = matched_sequence_indices.into_iter().collect();
        sorted_indices.sort_by_key(|&(start, _)| start);
        let all_matches = sorted_indices.into_iter()
            .map(|(start, len)| draws_sorted[start..start + len].iter().collect())
            .collect();

        // Return matches AND the parsed rules
        Ok((all_matches, rules))
    }


    fn timestamp_to_beijing_time(timestamp: u64) -> String {
         let timestamp_seconds = if timestamp > 10_000_000_000 { timestamp / 1000 } else { timestamp };
         let milliseconds = if timestamp > 10_000_000_000 { timestamp % 1000 } else { 0 };
         match Utc.timestamp_opt(timestamp_seconds as i64, 0) {
             chrono::LocalResult::Single(utc_time) => {
                 let beijing_timezone = FixedOffset::east_opt(8 * 3600).unwrap(); // UTC+8
                 let beijing_time: DateTime<FixedOffset> = utc_time.with_timezone(&beijing_timezone);
                 if milliseconds > 0 { format!("{}.{:03}", beijing_time.format("%Y-%m-%d %H:%M:%S"), milliseconds) }
                 else { beijing_time.format("%Y-%m-%d %H:%M:%S").to_string() }
             },
             _ => "時間格式錯誤".to_string(),
         }
    }
}


impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🏁 飛艇歷史數據查詢器");
            ui.horizontal(|ui| { ui.label("🔑 Cookie:"); ui.text_edit_singleline(&mut self.cookie); });
            ui.separator();
            ui.heading("選擇遊戲類型");
            ui.horizontal_wrapped(|ui| {
                let mut changed_game = false; let current_game = self.current_game;
                for game_type in GameType::all() {
                    if ui.radio_value(&mut self.current_game, game_type, game_type.to_name()).clicked() && self.current_game != current_game { changed_game = true; } }
                 if changed_game { self.draws.clear(); self.status = format!("已選擇 {}, 請點擊獲取數據", self.current_game.to_name()).into(); }
            });
            if ui.button("📥 獲取數據").clicked() { self.fetch_data(); }
            ui.label(&self.status);
            ui.separator();
            ui.horizontal(|ui| { ui.label("🔍 連續模式:"); ui.text_edit_singleline(&mut self.filter); });
            ui.label("💡 格式: 1單單23雙雙 (數字=位置, 23=位置2或3, 後跟模式)");
            ui.label("💡 模式: 單/雙/大/小 (大≥6, 小<6)");
            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // --- Updated Matching Logic Call and Display ---
                 if !self.filter.trim().is_empty() && !self.draws.is_empty() {
                     // Call the function which now returns Result<(matches, rules), error>
                     let find_result = self.find_matching_sequences(); // No argument needed

                     match find_result {
                         // Successfully parsed (even if no matches found)
                         Ok((all_matched, parsed_rules)) => {
                             if !all_matched.is_empty() {
                                 ui.heading(format!("🎯 找到 {} 組連續模式：", all_matched.len()));
                                 ui.add_space(5.0);

                                 for (group_idx, matched_group) in all_matched.iter().enumerate() {
                                     ui.push_id(group_idx, |ui| {
                                         let group_len = matched_group.len(); // Length of this specific match group

                                         for draw in matched_group {
                                             let beijing_time = Self::timestamp_to_beijing_time(draw.drawTime);
                                             let mut match_info = String::new();
                                             let mut added_positions = HashSet::new(); // Track positions per draw

                                             // Attempt to parse numbers for this draw
                                             let numbers_res: Result<Vec<u32>, _> = draw.result
                                                .split(',')
                                                .map(|s| s.trim().parse::<u32>())
                                                .collect();

                                            if let Ok(numbers) = numbers_res {
                                                // Iterate through the rules that were successfully parsed
                                                for rule_part in &parsed_rules {
                                                     // *** Only consider rules relevant to this group's length ***
                                                     if rule_part.pattern.len() == group_len {
                                                         for &pos in &rule_part.positions {
                                                            // Try to insert the position, only proceed if it's new for this draw
                                                             if added_positions.insert(pos) {
                                                                 if pos > 0 && pos <= numbers.len() {
                                                                     let num = numbers[pos - 1];
                                                                     let characteristic = determine_characteristic(num); // Use the new helper
                                                                     match_info.push_str(&format!("位{}:{} ", pos, characteristic));
                                                                 }
                                                             }
                                                         }
                                                     }
                                                 }
                                             } else {
                                                 // Handle error parsing numbers for this draw if needed
                                                 match_info.push_str("[結果解析錯誤] ")
                                             }


                                             ui.label(format!(
                                                 "✅ [{}] {} → {} ({})", // Add match_info here
                                                 beijing_time,
                                                 draw.drawNumber,
                                                 draw.result,
                                                 match_info.trim_end() // Trim trailing space
                                             ));
                                         } // End loop draw in matched_group

                                         if group_idx < all_matched.len() - 1 {
                                             ui.separator(); ui.add_space(3.0);
                                         }
                                     }); // End push_id
                                 } // End loop group_idx
                             } else {
                                 // Parsed OK, but no matches found
                                 ui.label("❌ 沒有找到符合連續模式的結果");
                             }
                         }
                         // Parsing failed
                         Err(e) => {
                             ui.colored_label(egui::Color32::RED, format!("❌ 模式錯誤: {:?}", e));
                         }
                     } // End match find_result
                 } else if !self.draws.is_empty() {
                     // Display all data if no filter
                     ui.heading("📋 全部數據：");
                     for draw in &self.draws {
                         let beijing_time = Self::timestamp_to_beijing_time(draw.drawTime);
                         ui.label(format!("📊 [{}] {} → {}", beijing_time, draw.drawNumber, draw.result));
                     }
                 } else {
                     if !self.status.contains("載入失敗") && !self.status.contains("抓取中"){ ui.label("請先獲取數據..."); }
                 }
                 // --- End Updated Display ---
            }); // End ScrollArea
        }); // End CentralPanel
    } // End update
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "飛艇記錄查詢器",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}