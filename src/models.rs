use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHours {
    pub id: Uuid,
    pub store_id: Uuid,
    pub weekday: Weekday,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub is_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBusinessHoursRequest {
    pub store_id: Uuid,
    pub weekday: Weekday,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    #[serde(default)]
    pub is_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateBusinessHoursRequest {
    pub store_id: Uuid,
    pub hours: Vec<BusinessHoursItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetUniformHoursRequest {
    pub store_id: Uuid,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    #[serde(default)]
    pub is_closed: bool,
    #[serde(default = "default_seven_days")]
    pub apply_weekdays: Vec<Weekday>,
}

fn default_seven_days() -> Vec<Weekday> {
    vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHoursItem {
    pub weekday: Weekday,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    #[serde(default)]
    pub is_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessStatusResponse {
    pub store_id: Uuid,
    pub store_name: String,
    pub is_open: bool,
    pub current_status: String,
    pub weekday: Weekday,
    pub current_time: NaiveTime,
    pub today_hours: Option<TodayHours>,
    pub next_open_time: Option<NaiveDateTime>,
    pub next_close_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayHours {
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub is_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHoursResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub weekday: Weekday,
    pub weekday_name: String,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub is_closed: bool,
}

impl BusinessHours {
    pub fn contains(&self, time: NaiveTime) -> bool {
        if self.is_closed {
            return false;
        }
        if self.close_time > self.open_time {
            time >= self.open_time && time < self.close_time
        } else {
            time >= self.open_time || time < self.close_time
        }
    }
}

pub fn weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "星期一",
        Weekday::Tue => "星期二",
        Weekday::Wed => "星期三",
        Weekday::Thu => "星期四",
        Weekday::Fri => "星期五",
        Weekday::Sat => "星期六",
        Weekday::Sun => "星期日",
    }
}

pub fn days_until_weekday(from: NaiveDate, target: Weekday) -> i64 {
    let from_weekday = from.weekday();
    let diff = (target.num_days_from_monday() as i64 - from_weekday.num_days_from_monday() as i64
        + 7)
        % 7;
    if diff == 0 {
        7
    } else {
        diff
    }
}
