# TaiL 数据库结构参考

本文档详细说明 TaiL 使用的 SQLite 数据库结构。

## 数据库位置

```
~/.local/share/tail/tail.db
```

可以通过环境变量覆盖：
```bash
export XDG_DATA_HOME=/custom/path
```

## 表结构

### window_events

存储窗口使用记录。

| 列名 | 类型 | 说明 | 约束 |
|------|------|------|------|
| id | INTEGER | 主键 | PRIMARY KEY AUTOINCREMENT |
| app_name | TEXT | 应用名称 | NOT NULL |
| window_title | TEXT | 窗口标题 | |
| start_time | TEXT | 开始时间 (ISO 8601) | NOT NULL |
| duration_secs | INTEGER | 持续时长（秒） | NOT NULL |
| is_afk | INTEGER | 是否 AFK (0/1) | DEFAULT 0 |

**索引：**
```sql
CREATE INDEX idx_window_events_time ON window_events(start_time);
CREATE INDEX idx_window_events_app ON window_events(app_name);
```

**示例数据：**
```sql
INSERT INTO window_events (app_name, window_title, start_time, duration_secs, is_afk)
VALUES ('code', 'tail - TaiL', '2024-01-15T10:30:00Z', 3600, 0);
```

### afk_events

存储空闲时段记录。

| 列名 | 类型 | 说明 | 约束 |
|------|------|------|------|
| id | INTEGER | 主键 | PRIMARY KEY AUTOINCREMENT |
| start_time | TEXT | 开始时间 (ISO 8601) | NOT NULL |
| end_time | TEXT | 结束时间 (ISO 8601) | |
| duration_secs | INTEGER | 持续时长（秒） | NOT NULL |

**索引：**
```sql
CREATE INDEX idx_afk_events_time ON afk_events(start_time);
```

**示例数据：**
```sql
INSERT INTO afk_events (start_time, end_time, duration_secs)
VALUES ('2024-01-15T12:00:00Z', '2024-01-15T12:15:00Z', 900);
```

### daily_goals

存储每日使用目标。

| 列名 | 类型 | 说明 | 约束 |
|------|------|------|------|
| id | INTEGER | 主键 | PRIMARY KEY AUTOINCREMENT |
| app_name | TEXT | 应用名称 | NOT NULL, UNIQUE |
| target_secs | INTEGER | 目标时长（秒） | NOT NULL |
| date | TEXT | 日期 (YYYY-MM-DD) | NOT NULL |

**索引：**
```sql
CREATE INDEX idx_daily_goals_date ON daily_goals(date);
```

**示例数据：**
```sql
INSERT INTO daily_goals (app_name, target_secs, date)
VALUES ('firefox', 7200, '2024-01-15');
```

## 常用查询

### 获取今日应用使用统计

```sql
SELECT
    app_name,
    SUM(duration_secs) as total_secs,
    SUM(duration_secs) / 3600.0 as total_hours,
    COUNT(*) as event_count
FROM window_events
WHERE date(start_time) = date('now', 'localtime')
GROUP BY app_name
ORDER BY total_secs DESC;
```

### 获取最近 N 条记录

```sql
SELECT * FROM window_events
ORDER BY start_time DESC
LIMIT 10;
```

### 按时间范围查询

```sql
-- 查询今日数据
SELECT * FROM window_events
WHERE date(start_time) = date('now', 'localtime');

-- 查询本周数据
SELECT * FROM window_events
WHERE start_time >= date('now', '-7 days', 'localtime');

-- 查询自定义时间范围
SELECT * FROM window_events
WHERE start_time BETWEEN '2024-01-01' AND '2024-01-31';
```

### 查询 AFK 时段

```sql
SELECT
    datetime(start_time) as start,
    datetime(end_time) as end,
    duration_secs / 60.0 as duration_minutes
FROM afk_events
ORDER BY start_time DESC
LIMIT 20;
```

### 统计总使用时长（排除 AFK）

```sql
SELECT SUM(duration_secs) as total_seconds
FROM window_events
WHERE is_afk = 0
  AND date(start_time) = date('now', 'localtime');
```

## 数据库维护

### 优化数据库

```bash
sqlite3 ~/.local/share/tail/tail.db "VACUUM;"
```

### 分析查询计划

```sql
EXPLAIN QUERY PLAN
SELECT * FROM window_events
WHERE app_name = 'code'
ORDER BY start_time DESC;
```

### 检查数据库完整性

```bash
sqlite3 ~/.local/share/tail/tail.db "PRAGMA integrity_check;"
```

### 清理旧数据

```bash
# 删除 30 天前的数据
sqlite3 ~/.local/share/tail/tail.db "
  DELETE FROM window_events WHERE date(start_time) < date('now', '-30 days');
  DELETE FROM afk_events WHERE date(start_time) < date('now', '-30 days');
  VACUUM;
"
```

## 数据库版本

当前数据库版本：**1.0**

### 变更历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2024-01-15 | 初始版本 |

### 迁移策略

未来版本将通过 SQL 迁移脚本更新数据库结构：

```sql
-- 示例迁移
ALTER TABLE window_events ADD COLUMN category TEXT;
CREATE INDEX idx_window_events_category ON window_events(category);
```
