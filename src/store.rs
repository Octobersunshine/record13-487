use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    BatchCreateBusinessHoursRequest, BusinessHours, BusinessHoursResponse,
    BusinessStatusResponse, CreateBusinessHoursRequest, Store, TodayHours, days_until_weekday,
    weekday_name,
};

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

#[derive(Debug, Default)]
pub struct AppStateInner {
    pub stores: RwLock<HashMap<Uuid, Store>>,
    pub business_hours: RwLock<HashMap<Uuid, Vec<BusinessHours>>>,
}

impl AppState {
    pub fn new() -> Self {
        let inner = Arc::new(AppStateInner::default());
        Self { inner }
    }

    pub async fn seed_demo_data(&self) {
        let store = Store {
            id: Uuid::new_v4(),
            name: "示例门店".to_string(),
            address: "示例地址 123 号".to_string(),
            created_at: Local::now().naive_local(),
        };

        let store_id = store.id;

        let weekdays = vec![
            (Weekday::Mon, NaiveTime::from_hms_opt(9, 0, 0).unwrap(), NaiveTime::from_hms_opt(21, 0, 0).unwrap(), false),
            (Weekday::Tue, NaiveTime::from_hms_opt(9, 0, 0).unwrap(), NaiveTime::from_hms_opt(21, 0, 0).unwrap(), false),
            (Weekday::Wed, NaiveTime::from_hms_opt(9, 0, 0).unwrap(), NaiveTime::from_hms_opt(21, 0, 0).unwrap(), false),
            (Weekday::Thu, NaiveTime::from_hms_opt(9, 0, 0).unwrap(), NaiveTime::from_hms_opt(21, 0, 0).unwrap(), false),
            (Weekday::Fri, NaiveTime::from_hms_opt(9, 0, 0).unwrap(), NaiveTime::from_hms_opt(22, 0, 0).unwrap(), false),
            (Weekday::Sat, NaiveTime::from_hms_opt(10, 0, 0).unwrap(), NaiveTime::from_hms_opt(22, 0, 0).unwrap(), false),
            (Weekday::Sun, NaiveTime::from_hms_opt(10, 0, 0).unwrap(), NaiveTime::from_hms_opt(20, 0, 0).unwrap(), false),
        ];

        let hours: Vec<BusinessHours> = weekdays
            .into_iter()
            .map(|(weekday, open, close, is_closed)| BusinessHours {
                id: Uuid::new_v4(),
                store_id,
                weekday,
                open_time: open,
                close_time: close,
                is_closed,
            })
            .collect();

        self.inner.stores.write().await.insert(store_id, store);
        self.inner.business_hours.write().await.insert(store_id, hours);
    }
}

pub async fn create_store(state: &AppState, name: String, address: String) -> Store {
    let store = Store {
        id: Uuid::new_v4(),
        name,
        address,
        created_at: Local::now().naive_local(),
    };
    let id = store.id;
    state.inner.stores.write().await.insert(id, store.clone());
    state
        .inner
        .business_hours
        .write()
        .await
        .insert(id, Vec::new());
    store
}

pub async fn get_store(state: &AppState, store_id: Uuid) -> Option<Store> {
    state.inner.stores.read().await.get(&store_id).cloned()
}

pub async fn add_business_hours(
    state: &AppState,
    req: CreateBusinessHoursRequest,
) -> Result<BusinessHoursResponse, String> {
    if state.inner.stores.read().await.get(&req.store_id).is_none() {
        return Err("门店不存在".to_string());
    }

    let hours = BusinessHours {
        id: Uuid::new_v4(),
        store_id: req.store_id,
        weekday: req.weekday,
        open_time: req.open_time,
        close_time: req.close_time,
        is_closed: req.is_closed,
    };

    let response = BusinessHoursResponse {
        id: hours.id,
        store_id: hours.store_id,
        weekday: hours.weekday,
        weekday_name: weekday_name(hours.weekday).to_string(),
        open_time: hours.open_time,
        close_time: hours.close_time,
        is_closed: hours.is_closed,
    };

    let mut store_hours = state.inner.business_hours.write().await;
    let store_hours_vec = store_hours.entry(req.store_id).or_insert_with(Vec::new);
    store_hours_vec.retain(|h| h.weekday != req.weekday);
    store_hours_vec.push(hours);

    Ok(response)
}

pub async fn batch_add_business_hours(
    state: &AppState,
    req: BatchCreateBusinessHoursRequest,
) -> Result<Vec<BusinessHoursResponse>, String> {
    if state.inner.stores.read().await.get(&req.store_id).is_none() {
        return Err("门店不存在".to_string());
    }

    let mut responses = Vec::new();
    let mut store_hours = state.inner.business_hours.write().await;
    let store_hours_vec = store_hours.entry(req.store_id).or_insert_with(Vec::new);

    for item in req.hours {
        let hours = BusinessHours {
            id: Uuid::new_v4(),
            store_id: req.store_id,
            weekday: item.weekday,
            open_time: item.open_time,
            close_time: item.close_time,
            is_closed: item.is_closed,
        };

        responses.push(BusinessHoursResponse {
            id: hours.id,
            store_id: hours.store_id,
            weekday: hours.weekday,
            weekday_name: weekday_name(hours.weekday).to_string(),
            open_time: hours.open_time,
            close_time: hours.close_time,
            is_closed: hours.is_closed,
        });

        store_hours_vec.retain(|h| h.weekday != item.weekday);
        store_hours_vec.push(hours);
    }

    Ok(responses)
}

pub async fn get_business_hours(
    state: &AppState,
    store_id: Uuid,
) -> Result<Vec<BusinessHoursResponse>, String> {
    if state.inner.stores.read().await.get(&store_id).is_none() {
        return Err("门店不存在".to_string());
    }

    let store_hours = state.inner.business_hours.read().await;
    let hours = store_hours.get(&store_id).cloned().unwrap_or_default();

    let weekday_order: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    let mut responses: Vec<BusinessHoursResponse> = hours
        .into_iter()
        .map(|h| BusinessHoursResponse {
            id: h.id,
            store_id: h.store_id,
            weekday: h.weekday,
            weekday_name: weekday_name(h.weekday).to_string(),
            open_time: h.open_time,
            close_time: h.close_time,
            is_closed: h.is_closed,
        })
        .collect();

    responses.sort_by(|a, b| {
        let idx_a = weekday_order.iter().position(|w| w == &a.weekday).unwrap_or(7);
        let idx_b = weekday_order.iter().position(|w| w == &b.weekday).unwrap_or(7);
        idx_a.cmp(&idx_b)
    });

    Ok(responses)
}

pub async fn get_business_status(
    state: &AppState,
    store_id: Uuid,
) -> Result<BusinessStatusResponse, String> {
    let store = state
        .inner
        .stores
        .read()
        .await
        .get(&store_id)
        .cloned()
        .ok_or_else(|| "门店不存在".to_string())?;

    let now = Local::now().naive_local();
    let current_weekday = now.weekday();
    let current_time = now.time();
    let current_date = now.date();

    let weekday_order: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];
    let current_idx = weekday_order
        .iter()
        .position(|w| w == &current_weekday)
        .unwrap();
    let prev_weekday = weekday_order[(current_idx + 6) % 7];
    let prev_date = current_date - Duration::days(1);

    let store_hours = state.inner.business_hours.read().await;
    let hours = store_hours.get(&store_id).cloned().unwrap_or_default();

    let today_hours_opt = hours.iter().find(|h| h.weekday == current_weekday);
    let prev_day_hours_opt = hours.iter().find(|h| h.weekday == prev_weekday);

    let today_hours = today_hours_opt.map(|h| TodayHours {
        open_time: h.open_time,
        close_time: h.close_time,
        is_closed: h.is_closed,
    });

    let mut is_open = false;
    let mut current_status = String::new();
    let mut next_close_time: Option<NaiveDateTime> = None;

    if let Some(prev_h) = prev_day_hours_opt {
        let is_overnight = prev_h.close_time < prev_h.open_time;
        if !prev_h.is_closed && is_overnight && current_time < prev_h.close_time {
            is_open = true;
            current_status = "营业中".to_string();
            next_close_time = Some(NaiveDateTime::new(current_date, prev_h.close_time));
        }
    }

    if !is_open {
        match today_hours_opt {
            Some(h) if !h.is_closed => {
                let within_hours = h.contains(current_time);
                if within_hours {
                    let close_dt = if h.close_time > h.open_time {
                        NaiveDateTime::new(current_date, h.close_time)
                    } else if current_time >= h.open_time {
                        NaiveDateTime::new(current_date + Duration::days(1), h.close_time)
                    } else {
                        NaiveDateTime::new(current_date, h.close_time)
                    };
                    is_open = true;
                    current_status = "营业中".to_string();
                    next_close_time = Some(close_dt);
                } else {
                    current_status = "休息中".to_string();
                }
            }
            Some(_) => current_status = "今日休息".to_string(),
            None => current_status = "未设置营业时间".to_string(),
        }
    } else {
        if today_hours_opt.is_none() {
            current_status = "营业中(凌晨时段)".to_string();
        }
    }

    let next_open_time = if is_open {
        None
    } else {
        find_next_open_time(
            &hours,
            current_weekday,
            prev_weekday,
            current_time,
            current_date,
            prev_date,
        )
    };

    Ok(BusinessStatusResponse {
        store_id: store.id,
        store_name: store.name,
        is_open,
        current_status,
        weekday: current_weekday,
        current_time,
        today_hours,
        next_open_time,
        next_close_time,
    })
}

fn find_next_open_time(
    hours: &[BusinessHours],
    current_weekday: Weekday,
    _prev_weekday: Weekday,
    current_time: NaiveTime,
    current_date: NaiveDate,
    _prev_date: NaiveDate,
) -> Option<NaiveDateTime> {
    let weekday_order: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    let today_hours = hours.iter().find(|h| h.weekday == current_weekday);
    if let Some(h) = today_hours {
        if !h.is_closed && current_time < h.open_time {
            return Some(NaiveDateTime::new(current_date, h.open_time));
        }
    }

    let current_idx = weekday_order
        .iter()
        .position(|w| w == &current_weekday)
        .unwrap();

    for i in 1..=7 {
        let idx = (current_idx + i) % 7;
        let weekday = weekday_order[idx];
        let days_offset = if i == 0 { 0 } else { i as i64 };
        let target_date = current_date + Duration::days(days_offset);

        if let Some(h) = hours.iter().find(|h| h.weekday == weekday) {
            if !h.is_closed {
                return Some(NaiveDateTime::new(target_date, h.open_time));
            }
        }
    }

    None
}
