# 分类视图改进计划

## 任务概述

根据用户需求，需要完成以下两个主要改进：

1. **替换文字 emoji 为 egui 特殊符号**：当前使用的是 Unicode emoji 字符，需要调研并使用 egui 文档中推荐的特殊符号
2. **完善分类视图的应用管理功能**：
   - 在分类视图中无法添加新应用到分类
   - 无法从分类中删除已有应用

## 问题分析

### 1. 当前 emoji 使用情况

查看 [`tail-core/src/models.rs:231-256`](tail-core/src/models.rs:231) 中的 `CATEGORY_ICONS` 常量：

```rust
pub const CATEGORY_ICONS: &[&str] = &[
    // 文件夹和文档
    "🗀", "🗁", "🗋", "🗐", "📋", "📌", "📎",
    // 图表和统计
    "📈", "📉", "📊",
    // ... 更多 Unicode emoji
];
```

**问题**：
- 使用的是 Unicode emoji 字符（如 🗀、📋 等）
- 这些字符依赖于系统字体支持，可能在某些环境下显示不正确
- egui 提供了内置的特殊符号，更可靠且渲染效果更好

### 2. 分类视图应用管理功能分析

查看 [`tail-gui/src/views/categories.rs`](tail-gui/src/views/categories.rs) 的当前实现：

**已有功能**：
- ✅ 添加新分类（第 124-128 行）
- ✅ 编辑分类（第 272-277 行）
- ✅ 删除分类（第 279-284 行）
- ✅ 管理应用分类对话框（第 132-134 行）
- ✅ 从分类中移除应用（第 333-336 行）

**缺失功能**：
- ❌ 在"管理应用分类"对话框中，只能为已有应用分配分类，无法添加新应用
- ❌ 虽然有移除按钮（✕），但用户可能不清楚这是删除功能

**根本原因**：
- 应用列表来自 `get_all_app_names()` 方法（第 86-88 行），只返回数据库中已记录的应用
- 没有提供手动输入新应用名称的功能
- UI 上删除功能已存在，但可能需要更明确的提示

## 解决方案设计

### 方案 1：使用 egui 特殊符号替换 emoji

根据 egui 文档 https://docs.rs/egui/latest/egui/special_emojis/index.html，egui 提供了一组特殊符号常量。

**实施步骤**：

1. **调研 egui 特殊符号**
   - 查看 `egui::special_emojis` 模块提供的符号
   - 选择适合分类图标的符号

2. **更新 CATEGORY_ICONS 常量**
   - 在 [`tail-core/src/models.rs`](tail-core/src/models.rs) 中替换现有的 Unicode emoji
   - 使用 egui 提供的特殊符号常量

3. **可用的 egui 特殊符号示例**：
   ```rust
   // egui 提供的特殊符号（来自文档）
   pub const GITHUB: &str = "";
   pub const TWITTER: &str = "";
   pub const MEDIUM: &str = "";
   // ... 等等
   ```

**注意**：需要实际查看 egui 文档确定可用符号列表。

### 方案 2：完善分类视图的应用管理功能

#### 2.1 添加新应用到分类

**设计思路**：
在"管理应用分类"对话框中添加一个输入框，允许用户手动输入新应用名称。

**UI 改进**：
```
┌─────────────────────────────────────┐
│ 管理应用分类                         │
├─────────────────────────────────────┤
│ 选择应用:                            │
│ ┌─────────────────────────────────┐ │
│ │ [应用列表 - 滚动区域]            │ │
│ └─────────────────────────────────┘ │
│                                     │
│ 或手动输入新应用:                    │
│ [_____________________________]     │
│ [添加新应用] 按钮                    │
│                                     │
│ 为 'xxx' 选择分类:                   │
│ ☑ 🗀 工作                           │
│ ☐ 📊 娱乐                           │
│                                     │
│ [保存] [取消]                        │
└─────────────────────────────────────┘
```

**实现要点**：
1. 添加新的状态字段：`new_app_name_input: String`
2. 在对话框中添加文本输入框
3. 添加"添加新应用"按钮，点击后将输入的应用名添加到选择列表
4. 验证应用名不为空

#### 2.2 改进删除应用的 UI 提示

**当前实现**：
- 第 333-336 行有删除按钮 `✕`
- 有 hover 提示："从此分类中移除"

**改进建议**：
- 保持当前实现，已经足够清晰
- 可选：添加确认对话框（如果用户反馈需要）

## 数据库支持

查看 [`tail-core/src/db.rs`](tail-core/src/db.rs) 的相关方法：

**已有支持**：
- ✅ `add_app_to_category()` - 添加应用到分类（第 480-487 行）
- ✅ `remove_app_from_category()` - 从分类移除应用（第 490-497 行）
- ✅ `set_app_categories()` - 设置应用的分类（第 641-659 行）
- ✅ `get_all_app_names()` - 获取所有应用名称（第 662-672 行）

**结论**：数据库层已经完全支持所需功能，无需修改。

## 实施计划

### 阶段 1：调研 egui 特殊符号

- [ ] 访问 egui 文档或查看源码
- [ ] 列出所有可用的特殊符号
- [ ] 选择适合作为分类图标的符号
- [ ] 创建新的符号映射表

### 阶段 2：替换 emoji 为 egui 特殊符号

**文件修改**：
- [ ] [`tail-core/src/models.rs`](tail-core/src/models.rs)
  - 更新 `CATEGORY_ICONS` 常量
  - 使用 egui 特殊符号替换 Unicode emoji

**测试**：
- [ ] 验证图标在 GUI 中正确显示
- [ ] 确保现有分类的图标仍然可用

### 阶段 3：添加新应用功能

**文件修改**：
- [ ] [`tail-gui/src/views/categories.rs`](tail-gui/src/views/categories.rs)
  - 在 `CategoriesView` 结构体中添加 `new_app_name_input: String` 字段
  - 在 `new()` 方法中初始化该字段
  - 修改 `show_assign_apps_dialog()` 方法：
    - 添加"手动输入新应用"部分的 UI
    - 添加输入框和"添加新应用"按钮
    - 实现添加逻辑：验证输入并添加到应用列表

**具体实现位置**：
- 在第 527-615 行的 `show_assign_apps_dialog()` 方法中
- 在应用列表滚动区域后添加新的 UI 部分

### 阶段 4：测试和验证

- [ ] 测试添加新应用功能
  - 输入新应用名称
  - 为新应用分配分类
  - 验证保存后数据正确
- [ ] 测试删除应用功能
  - 从分类中移除应用
  - 验证数据更新正确
- [ ] 测试 egui 特殊符号显示
  - 在不同主题下测试
  - 验证符号清晰可见

## 技术细节

### egui 特殊符号使用方式

根据 egui 文档，特殊符号通常定义在 `egui::special_emojis` 模块中：

```rust
use egui::special_emojis;

// 使用示例
let icon = special_emojis::GITHUB;
ui.label(icon);
```

**注意**：需要实际查看 egui 0.x 版本的文档确定可用符号。

### 添加新应用的实现伪代码

```rust
// 在 CategoriesView 结构体中添加
new_app_name_input: String,

// 在 show_assign_apps_dialog 中添加
ui.add_space(self.theme.spacing);
ui.separator();
ui.label("或手动输入新应用:");
ui.horizontal(|ui| {
    ui.text_edit_singleline(&mut self.new_app_name_input);
    if ui.button("添加新应用").clicked() {
        if !self.new_app_name_input.trim().is_empty() {
            let new_app = self.new_app_name_input.trim().to_string();
            if !self.all_apps.contains(&new_app) {
                self.all_apps.push(new_app.clone());
                self.all_apps.sort();
            }
            self.selected_app_name = Some(new_app);
            self.new_app_name_input.clear();
            // 加载该应用当前的分类
            let current_categories = repo.get_app_categories(&self.selected_app_name.as_ref().unwrap())
                .unwrap_or_default();
            self.selected_category_ids = current_categories.iter()
                .filter_map(|c| c.id)
                .collect();
        }
    }
});
```

## 风险和注意事项

1. **egui 特殊符号兼容性**
   - 需要确认项目使用的 egui 版本支持哪些特殊符号
   - 某些符号可能在不同平台上显示效果不同

2. **数据一致性**
   - 添加新应用时，应用可能还没有任何使用记录
   - 这是正常的，用户可以提前为应用分配分类

3. **用户体验**
   - 添加新应用后，应该自动选中该应用
   - 清空输入框，准备下次输入

## 后续优化建议

1. **应用名称自动补全**
   - 可以从系统中扫描已安装的应用
   - 提供自动补全功能

2. **批量操作**
   - 允许一次为多个应用分配相同分类
   - 批量删除应用

3. **分类图标自定义**
   - 除了预设图标，允许用户输入自定义符号
   - 提供图标预览功能

## 总结

本计划涵盖了两个主要改进：

1. **替换 emoji 为 egui 特殊符号**：提高图标显示的可靠性和一致性
2. **完善应用管理功能**：允许用户在分类视图中添加新应用

这些改进将显著提升用户体验，使分类管理功能更加完整和易用。
