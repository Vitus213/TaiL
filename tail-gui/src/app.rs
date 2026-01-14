//! TaiL GUI - egui 应用

use chrono::{DateTime, Utc, Duration as ChronoDuration};
use tail_core::{DbConfig, Repository, AppUsage, DailyGoal};
use tail_core::models::TimeRange;
use std::sync::Arc;

/// TaiL GUI 应用
pub struct TaiLApp {
    /// 当前视图
    current_view: View,

    /// 选中的时间范围
    time_range: TimeRange,

    /// 数据库仓库
    repo: Arc<Repository>,

    /// 应用使用数据缓存
    app_usage_cache: Vec<AppUsage>,

    /// 每日目标缓存
    daily_goals_cache: Vec<DailyGoal>,

    /// 上次刷新时间
    last_refresh: Option<DateTime<Utc>>,

    /// 新建目标对话框状态
    show_add_goal_dialog: bool,new_goal_app_name: String,
    new_goal_max_minutes: i32,
}

/// 视图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Dashboard,
    Statistics,
    Settings,
}

impl TaiLApp {
    /// 创建新的应用实例
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = DbConfig::default();
        tracing::info!("初始化数据库，路径: {}", config.path);
        
        let repo = Repository::new(&config)
            .expect("Failed to initialize database");
        
        tracing::info!("TaiL GUI 应用初始化成功");

        Self {
            current_view: View::Dashboard,
            time_range: TimeRange::Today,
            repo: Arc::new(repo),
            app_usage_cache: Vec::new(),
            daily_goals_cache: Vec::new(),
            last_refresh: None,
            show_add_goal_dialog: false,
            new_goal_app_name: String::new(),
            new_goal_max_minutes: 60,
        }
    }

    /// 刷新数据
    fn refresh_data(&mut self) {
        let now = Utc::now();
        // 每2秒刷新一次（更频繁的更新）
        if let Some(last) = self.last_refresh {
            let elapsed = now.signed_duration_since(last).num_seconds();
            if elapsed < 2 {
                tracing::debug!("跳过刷新，距离上次刷新仅 {} 秒", elapsed);
                return;
            }
        }

        let (start, end) = self.get_time_range_bounds();
        tracing::info!("开始刷新数据，时间范围: {} 到 {}", start, end);
        
        // 刷新应用使用数据
        match self.repo.get_app_usage(start, end) {
            Ok(usage) => {
                tracing::info!("成功获取 {} 条应用使用记录", usage.len());
                if !usage.is_empty() {
                    tracing::debug!("前3条记录: {:?}", &usage[..usage.len().min(3)]);
                }
                self.app_usage_cache = usage;
            }
            Err(e) => {
                tracing::error!("获取应用使用数据失败: {}", e);
            }
        }

        // 刷新每日目标
        match self.repo.get_daily_goals() {
            Ok(goals) => {
                tracing::info!("成功获取 {} 条每日目标", goals.len());
                self.daily_goals_cache = goals;
            }
            Err(e) => {
                tracing::error!("获取每日目标失败: {}", e);
            }
        }

        self.last_refresh = Some(now);
        tracing::info!("数据刷新完成");
    }

    /// 获取时间范围的开始和结束时间
    fn get_time_range_bounds(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        let bounds = match self.time_range {
            TimeRange::Today => {
                let today_start = now.date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                (today_start, now)
            }
            TimeRange::Yesterday => {
                let yesterday = now - ChronoDuration::days(1);
                let yesterday_start = yesterday.date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let yesterday_end = yesterday.date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc();
                (yesterday_start, yesterday_end)
            }
            TimeRange::Last7Days => {
                let week_ago = now - ChronoDuration::days(7);
                (week_ago, now)
            }
            TimeRange::Last30Days => {
                let month_ago = now - ChronoDuration::days(30);
                (month_ago, now)
            }
            TimeRange::Custom(start, end) => (start, end),
        };
        tracing::debug!("时间范围边界: {:?} 到 {:?}", bounds.0, bounds.1);
        bounds
    }

    /// 格式化时长（秒转为可读格式）
    fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

impl eframe::App for TaiLApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 请求持续重绘以保持数据更新
        ctx.request_repaint();
        
        // 顶部导航栏
        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("TaiL");
                ui.separator();

                if ui.selectable_label(self.current_view == View::Dashboard, "仪表板").clicked() {
                    self.current_view = View::Dashboard;
                }
                if ui.selectable_label(self.current_view == View::Statistics, "统计").clicked() {
                    self.current_view = View::Statistics;
                }
                if ui.selectable_label(self.current_view == View::Settings, "设置").clicked() {
                    self.current_view = View::Settings;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("退出").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Dashboard => self.show_dashboard(ui),
                View::Statistics => self.show_statistics(ui),
                View::Settings => self.show_settings(ui),
            }
        });
    }
}

impl TaiLApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 今日使用统计");
        ui.add_space(10.0);

        // 刷新数据
        self.refresh_data();

        // 总使用时间
        let total_seconds: i64 = self.app_usage_cache.iter()
            .map(|u| u.total_seconds)
            .sum();
        
        tracing::debug!("仪表板显示: {} 条记录，总时长 {} 秒",
            self.app_usage_cache.len(), total_seconds);

        ui.horizontal(|ui| {
            ui.label("总使用时间:");
            ui.strong(Self::format_duration(total_seconds));
        });

        ui.add_space(10.0);ui.separator();
        ui.add_space(10.0);

        // 应用使用列表
        ui.heading("应用使用排行");
        ui.add_space(10.0);

        if self.app_usage_cache.is_empty() {
            ui.label("暂无数据");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, usage) in self.app_usage_cache.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", idx + 1));
                        ui.label(&usage.app_name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(Self::format_duration(usage.total_seconds));
                            
                            // 显示进度条
                            if total_seconds > 0 {
                                let percentage = (usage.total_seconds as f32 / total_seconds as f32) * 100.0;
                                ui.label(format!("{:.1}%", percentage));
                            }
                        });
                    });
                    
                    // 进度条
                    if total_seconds > 0 {
                        let fraction = usage.total_seconds as f32 / total_seconds as f32;
                        ui.add(egui::ProgressBar::new(fraction).show_percentage());
                    }
                    
                    ui.add_space(5.0);
                }
            });
        }
    }

    fn show_statistics(&mut self, ui: &mut egui::Ui) {
        ui.heading("📈 详细统计");
        ui.add_space(10.0);

        // 时间范围选择
        ui.horizontal(|ui| {
            ui.label("时间范围:");
            if ui.selectable_label(matches!(self.time_range, TimeRange::Today), "今天").clicked() {
                self.time_range = TimeRange::Today;self.last_refresh = None; // 强制刷新
            }
            if ui.selectable_label(matches!(self.time_range, TimeRange::Yesterday), "昨天").clicked() {
                self.time_range = TimeRange::Yesterday;
                self.last_refresh = None;
            }
            if ui.selectable_label(matches!(self.time_range, TimeRange::Last7Days), "最近7天").clicked() {
                self.time_range = TimeRange::Last7Days;
                self.last_refresh = None;
            }
            if ui.selectable_label(matches!(self.time_range, TimeRange::Last30Days), "最近30天").clicked() {
                self.time_range = TimeRange::Last30Days;
                self.last_refresh = None;
            }});

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // 刷新数据
        self.refresh_data();

        // 显示统计表格
        if self.app_usage_cache.is_empty() {
            ui.label("所选时间范围内暂无数据");
        } else {
            use egui_extras::{TableBuilder, Column};
            
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(40.0))  // 排名
                .column(Column::remainder())   // 应用名称
                .column(Column::exact(100.0))  // 使用时长
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("#");
                    });
                    header.col(|ui| {
                        ui.heading("应用");
                    });
                    header.col(|ui| {
                        ui.heading("时长");
                    });
                })
                .body(|mut body| {
                    for (idx, usage) in self.app_usage_cache.iter().enumerate() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{}", idx + 1));
                            });
                            row.col(|ui| {
                                ui.label(&usage.app_name);
                            });
                            row.col(|ui| {
                                ui.label(Self::format_duration(usage.total_seconds));
                            });
                        });
                    }
                });
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ 设置");
        ui.add_space(10.0);

        ui.label("每日目标设置");
        ui.add_space(10.0);

        //刷新每日目标
        if self.daily_goals_cache.is_empty() {
            if let Ok(goals) = self.repo.get_daily_goals() {
                self.daily_goals_cache = goals;
            }
        }

        // 显示现有目标
        let mut goals_to_delete = Vec::new();
        
        for goal in &self.daily_goals_cache {
            ui.horizontal(|ui| {
                ui.label(&goal.app_name);
                ui.label(format!("最多{} 分钟", goal.max_minutes));
                
                if ui.button("🗑").clicked() {
                    goals_to_delete.push(goal.app_name.clone());
                }
            });
        }

        // 删除标记的目标
        for app_name in goals_to_delete {
            if let Ok(()) = self.repo.delete_daily_goal(&app_name) {
                self.daily_goals_cache.retain(|g| g.app_name != app_name);
            }
        }

        ui.add_space(10.0);

        // 添加新目标按钮
        if ui.button("➕ 添加新目标").clicked() {
            self.show_add_goal_dialog = true;
        }

        // 新建目标对话框
        if self.show_add_goal_dialog {
            egui::Window::new("添加每日目标")
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.label("应用名称:");
                    ui.text_edit_singleline(&mut self.new_goal_app_name);
                    
                    ui.label("最大使用分钟数:");
                    ui.add(egui::Slider::new(&mut self.new_goal_max_minutes, 1..=480));

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            if !self.new_goal_app_name.is_empty() {
                                let goal = DailyGoal {
                                    id: None,
                                    app_name: self.new_goal_app_name.clone(),
                                    max_minutes: self.new_goal_max_minutes,notify_enabled: true,
                                };
                                if let Ok(_) = self.repo.upsert_daily_goal(&goal) {
                                    self.daily_goals_cache.push(goal);
                                    self.new_goal_app_name.clear();
                                    self.new_goal_max_minutes = 60;
                                    self.show_add_goal_dialog = false;
                                }
                            }
                        }
                        if ui.button("取消").clicked() {
                            self.new_goal_app_name.clear();
                            self.show_add_goal_dialog = false;
                        }
                    });
                });
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // 数据库路径信息
        ui.label("数据库位置:");
        let config = DbConfig::default();
        ui.small(&config.path);}
}
