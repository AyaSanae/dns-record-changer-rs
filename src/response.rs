use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DescribeRecordListResponse {
    #[serde(rename = "Response")]
    pub response: RecordListResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecordListResponse {
    #[serde(rename = "RequestId")]
    pub request_id: String,

    #[serde(rename = "RecordCountInfo")]
    pub record_count_info: RecordCountInfo,

    #[serde(rename = "RecordList")]
    pub record_list: Vec<Record>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecordCountInfo {
    #[serde(rename = "SubdomainCount")]
    pub subdomain_count: i32,

    #[serde(rename = "ListCount")]
    pub list_count: i32,

    #[serde(rename = "TotalCount")]
    pub total_count: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    #[serde(rename = "RecordId")]
    pub record_id: i64,

    #[serde(rename = "Value")]
    pub value: String,

    #[serde(rename = "Status")]
    pub status: String,

    #[serde(rename = "UpdatedOn")]
    pub updated_on: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Line")]
    pub line: String,

    #[serde(rename = "LineId")]
    pub line_id: String,

    #[serde(rename = "Type")]
    pub record_type: String,

    #[serde(rename = "Weight")]
    pub weight: Option<i32>,

    #[serde(rename = "MonitorStatus")]
    pub monitor_status: String,

    #[serde(rename = "Remark")]
    pub remark: String,

    #[serde(rename = "TTL")]
    pub ttl: i64,

    #[serde(rename = "MX")]
    pub mx: i32,

    #[serde(rename = "DefaultNS")]
    pub default_ns: bool,
}

impl fmt::Display for DescribeRecordListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DNS 记录列表响应:")?;
        writeln!(f, "{}", self.response)
    }
}

impl fmt::Display for RecordListResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "请求 ID: {}", self.request_id)?;
        writeln!(f, "{}", self.record_count_info)?;
        writeln!(f, "记录列表 (共 {} 条):", self.record_list.len())?;

        for (i, record) in self.record_list.iter().enumerate() {
            writeln!(f, "\n记录 #{}:", i + 1)?;
            writeln!(f, "{}", record)?;
        }

        Ok(())
    }
}

impl fmt::Display for RecordCountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "记录统计信息:")?;
        writeln!(f, "  子域名数量: {}", self.subdomain_count)?;
        writeln!(f, "  列表数量: {}", self.list_count)?;
        writeln!(f, "  总数量: {}", self.total_count)
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  记录 ID: {}", self.record_id)?;
        writeln!(f, "  名称: {}", self.name)?;
        writeln!(f, "  类型: {}", self.record_type)?;
        writeln!(f, "  值: {}", self.value)?;
        writeln!(f, "  TTL: {}", self.ttl)?;
        writeln!(f, "  线路: {}", self.line)?;
        writeln!(f, "  状态: {}", self.status)?;
        writeln!(f, "  监控状态: {}", self.monitor_status)?;
        writeln!(f, "  最后更新: {}", self.updated_on)?;

        if let Some(weight) = self.weight {
            writeln!(f, "  权重: {}", weight)?;
        }

        if self.record_type == "MX" {
            writeln!(f, "  MX 优先级: {}", self.mx)?;
        }

        if !self.remark.is_empty() {
            writeln!(f, "  备注: {}", self.remark)?;
        }

        write!(f, "  默认 NS: {}", self.default_ns)
    }
}

pub fn deserialize_describe_record_list_response(
    json_str: &str,
) -> Result<DescribeRecordListResponse, serde_json::Error> {
    serde_json::from_str(json_str)
}
