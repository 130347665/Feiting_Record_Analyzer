# 🏁 飛艇記錄查詢器

這是一款使用 Rust + egui + reqwest 打造的桌面 GUI 工具，專門用於查詢與分析各類飛艇／賽車遊戲的歷史開獎紀錄。支援 Cookie 登入、連續開獎模式匹配、繁體中文介面顯示。

---

## 🔧 功能特色

- 🎮 支援多種遊戲類型：
  - SG飛艇
  - 幸運飛艇
  - 急速飛艇
  - 急速賽車
- 🔐 自訂輸入 Cookie 進行授權
- 📥 一鍵抓取 `https://www.15678.vip` 開獎資料
- 🔍 支援連續模式查詢（如：`單單雙`、`大小小`）
- 🕒 自動轉換為北京時間（支援毫秒顯示）
- 🖼️ 繁體中文介面，使用微軟正黑體字型顯示

---

## 📦 建置方式

### 1. 安裝前置工具

請確認你已安裝：

- Rust 工具鏈（推薦使用 [https://rustup.rs](https://rustup.rs) 安裝）
- Cargo 套件管理工具（隨 Rust 安裝）
- Windows 字型路徑：`C:/Windows/Fonts/msyh.ttc`（微軟正黑體）

### 2. Clone 並建置專案

```bash
git clone https://your-repo-url
cd your-project-folder
cargo build --release
```

### 3. 執行應用程式

```bash
cargo run --release
```

或直接執行生成的 .exe 檔案。

#### 🖥️ 操作說明

1. 啟動後，請先輸入你的 登入 Cookie。

2. 選擇一個遊戲（預設為 SG飛艇）。

3. 點擊「📥 獲取數據」。

4. 若要查找連續模式，可輸入模式（例如：單雙單、大小大）。

5. 滾動查看所有匹配結果，包含期數、開獎結果與首號條件。


#### 📌 注意事項

網頁來源為 https://www.15678.vip 

需具備有效 Cookie 權限才能正常抓取數據。

本工具僅供學術與個人使用，請勿用於非法用途。

#### 📜 License
本專案採用 MIT 授權。歡迎自由使用與修改。
