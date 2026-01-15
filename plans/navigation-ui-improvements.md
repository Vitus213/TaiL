# 导航UI改进计划

## 用户反馈

用户反馈当前实现存在以下问题：
1. 没有显示年视图
2. 默认应该从今天开始（小时视图），而不是年视图
3. 缺少月份和周份的视图
4. 需要"本周"快捷按钮
5. 按钮布局需要调整：主按钮（今天、本周、昨天）+ 次要按钮（当月、当年）

## 当前实现分析

### 1. 默认导航状态
**文件**: `tail-gui/src/app.rs:93-95`
```rust
let current_year = Local::now().year();
let navigation_state = TimeNavigationState::new(current_year);
```
- 当前默认为年视图
- **需要改为**：默认为今天的小时视图

### 2. 导航控制器按钮
**文件**: `tail-gui/src/components/time_navigation.rs:24-73`
- 当前按钮：返回、今天、昨天
- **需要改为**：
  - 主按钮行：今天、本周、昨天
  - 次要按钮行：当月、当年
  - 返回按钮保留

### 3. TimeNavigationState 方法
**文件**: `tail-core/src/models.rs:90-164`
- 已有方法：`go_to_today()`, `go_to_yesterday()`
- **需要添加**：
  - `go_to_this_week()` - 跳转到本周（周视图）
  - `go_to_this_month()` - 跳转到当月（月视图）
  - `go_to_this_year()` - 跳转到当年（年视图）

### 4. 聚合逻辑
**文件**: `tail-gui/src/views/aggregation.rs`
需要确保各个层级的聚合逻辑正确：
- 年视图：显示12个月的数据
- 月视图：显示该月所有周的数据
- 周视图：显示7天的数据
- 天视图：显示24小时的数据

## 实现步骤

### 步骤1：修改 TimeNavigationState 默认行为
**文件**: `tail-core/src/models.rs`

添加新方法：
```rust
/// 跳转到本周（周视图）
pub fn go_to_this_week(&mut self, year: i32, month: u32, week: u32) {
    self.selected_year = year;
    self.selected_month = Some(month);
    self.selected_week = Some(week);
    self.selected_day = None;
    self.level = TimeNavigationLevel::Day; // 周视图显示7天
}

/// 跳转到当月（月视图）
pub fn go_to_this_month(&mut self, year: i32, month: u32) {
    self.selected_year = year;
    self.selected_month = Some(month);
    self.selected_week = None;
    self.selected_day = None;
    self.level = TimeNavigationLevel::Week; // 月视图显示周
}

/// 跳转到当年（年视图）
pub fn go_to_this_year(&mut self, year: i32) {
    self.selected_year = year;
    self.selected_month = None;
    self.selected_week = None;
    self.selected_day = None;
    self.level = TimeNavigationLevel::Month; // 年视图显示月
}
```

修改 `new()` 方法，默认跳转到今天：
```rust
pub fn new(current_year: i32) -> Self {
    let now = chrono::Local::now();
    let mut state = Self {
        level: TimeNavigationLevel::Hour,
        selected_year: now.year(),
        selected_month: Some(now.month()),
        selected_week: None,
        selected_day: Some(now.day()),
    };
    state
}
```

### 步骤2：修改 app.rs 初始化
**文件**: `tail-gui/src/app.rs:93-95`

```rust
// 初始化导航状态为今天的小时视图
let now = Local::now();
let navigation_state = TimeNavigationState::new(now.year());
```

### 步骤3：更新导航控制器UI
**文件**: `tail-gui/src/components/time_navigation.rs`

修改 `show()` 方法返回值和UI布局：
```rust
/// 显示导航控制器
/// 返回：(返回, 今天, 本周, 昨天, 当月, 当年)
pub fn show(&self, ui: &mut Ui) -> (bool, bool, bool, bool, bool, bool) {
    let mut go_back = false;
    let mut go_today = false;
    let mut go_this_week = false;
    let mut go_yesterday = false;
    let mut go_this_month = false;
    let mut go_this_year = false;

    ui.vertical(|ui| {
        // 第一行：面包屑和返回按钮
        ui.horizontal(|ui| {
            ui.label(format!("📍 {}", self.state.get_breadcrumb()));
            ui.add_space(16.0);
            if ui.button("⬅ 返回").clicked() {
                go_back = true;
            }
        });

        ui.add_space(8.0);

        // 第二行：主要快捷按钮
        ui.horizontal(|ui| {
            if ui.button("📅 今天").clicked() {
                go_today = true;
            }
            if ui.button("📆 本周").clicked() {
                go_this_week = true;
            }
            if ui.button("📆 昨天").clicked() {
                go_yesterday = true;
            }
        });

        ui.add_space(4.0);

        // 第三行：次要快捷按钮
        ui.horizontal(|ui| {
            if ui.button("📅 当月").clicked() {
                go_this_month = true;
            }
            if ui.button("📅 当年").clicked() {
                go_this_year = true;
            }
        });
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    (go_back, go_today, go_this_week, go_yesterday, go_this_month, go_this_year)
}
```

### 步骤4：更新 statistics.rs 处理导航事件
**文件**: `tail-gui/src/views/statistics.rs:55-71`

```rust
// 时间导航控制器
let controller = TimeNavigationController::new(self.navigation_state, self.theme);
let (go_back, go_today, go_this_week, go_yesterday, go_this_month, go_this_year) = controller.show(ui);

// 处理导航事件
if go_back {
    self.navigation_state.go_back();
    new_time_range = Some(self.navigation_state.to_time_range());
} else if go_today {
    let now = Local::now();
    self.navigation_state.go_to_today(now.year(), now.month(), now.day());
    new_time_range = Some(self.navigation_state.to_time_range());
} else if go_this_week {
    let now = Local::now();
    // 计算当前是本月第几周
    let week = calculate_week_of_month(&now);
    self.navigation_state.go_to_this_week(now.year(), now.month(), week);
    new_time_range = Some(self.navigation_state.to_time_range());
} else if go_yesterday {
    let yesterday = Local::now() - chrono::Duration::days(1);
    self.navigation_state.go_to_yesterday(yesterday.year(), yesterday.month(), yesterday.day());
    new_time_range = Some(self.navigation_state.to_time_range());
} else if go_this_month {
    let now = Local::now();
    self.navigation_state.go_to_this_month(now.year(), now.month());
    new_time_range = Some(self.navigation_state.to_time_range());
} else if go_this_year {
    let now = Local::now();
    self.navigation_state.go_to_this_year(now.year());
    new_time_range = Some(self.navigation_state.to_time_range());
}
```

需要添加辅助函数计算周数：
```rust
fn calculate_week_of_month(date: &chrono::DateTime<Local>) -> u32 {
    use chrono::Datelike;
    let first_day = date.date_naive().with_day(1).unwrap();
    let first_weekday = first_day.weekday().num_days_from_monday();
    let day_of_month = date.day();
    ((day_of_month + first_weekday - 1) / 7) + 1
}
```

### 步骤5：验证聚合逻辑
**文件**: `tail-gui/src/views/aggregation.rs`

确保 `aggregate()` 方法正确处理所有层级：
- `TimeNavigationLevel::Month`（年视图）：聚合12个月
- `TimeNavigationLevel::Week`（月视图）：聚合该月的周
- `TimeNavigationLevel::Day`（周视图）：聚合7天
- `TimeNavigationLevel::Hour`（天视图）：聚合24小时

## 测试计划

1. **默认视图测试**
   - 启动应用，验证默认显示今天的24小时视图
   - 验证面包屑显示正确的日期

2. **快捷按钮测试**
   - 点击"本周"，验证显示本周7天
   - 点击"当月"，验证显示当月所有周
   - 点击"当年"，验证显示12个月
   - 点击"今天"，验证返回今天的小时视图
   - 点击"昨天"，验证显示昨天的小时视图

3. **导航测试**
   - 从今天点击柱形图下钻，验证能正确返回
   - 从年视图逐级下钻到小时视图
   - 使用返回按钮逐级返回

4. **数据显示测试**
   - 验证每个层级的柱形图数量正确
   - 验证柱形图标签正确
   - 验证数据聚合正确

## 注意事项

1. 周数计算需要考虑月初不是周一的情况
2. 面包屑显示需要根据层级动态调整
3. 返回按钮在最顶层（年视图）应该禁用或隐藏
4. 确保所有时间计算使用本地时区
