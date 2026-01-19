use std::str::FromStr;
use std::time::Duration;
use std::{env, net::Ipv6Addr};

use clap::Parser;
use dns_rs::TencentCloudClient;
use dns_rs::response::deserialize_describe_record_list_response;
use serde_json::json;
use tokio::time;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long = "ifacename", value_name = "INTERFACE")]
    interface: String,
    #[arg(short = 'd', long = "domain", value_name = "DOMAIN")]
    domain: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    log::info!("配置环境中...");

    let mut interval = time::interval(Duration::from_secs(3600));

    let args = Args::parse();

    let secret_id = env::var("DNSPOD_SECRET_ID")
        .map_err(|e| format!("从环境变量获取secret_id失败: {},请设置DNSPOD_SECRET_ID ", e))?;
    let secret_key = env::var("DNSPOD_SECRET_KEY").map_err(|e| {
        format!(
            "从环境变量获取secret_key失败: {},请设置DNSPOD_SECRET_KEY ",
            e
        )
    })?;

    let domain = args.domain;
    let version = "2021-03-23";
    let client = TencentCloudClient::new(
        "dnspod",
        "dnspod.tencentcloudapi.com",
        "",
        &secret_id,
        &secret_key,
    )?;

    log::info!("环境配置完毕,开始运行...");
    loop {
        let mut action = "DescribeRecordList";
        let payload = json!({
            "Domain": domain
        });
        let mut ipv6_record = vec![];
        match client.request(action, version, payload).await {
            Ok(response) => {
                let response = deserialize_describe_record_list_response(&response)?;
                ipv6_record = response
                    .response
                    .record_list
                    .into_iter()
                    .filter(|record| record.record_type == "AAAA")
                    .collect();
            }
            Err(e) => {
                log::error!("Error: {}", e);
            }
        }

        log::debug!("ipv6_record: {:?}", ipv6_record);

        let local_ipv6_set = get_ipv6_set_with_iface(&args.interface).await?;
        if ipv6_record[0].value != local_ipv6_set[0].to_string() {
            log::warn!(
                "IPV6地址记录变更! \n记录值:{}\n本地值:{}",
                ipv6_record[0].value,
                local_ipv6_set[0]
            );
            log::warn!("开始请求修改记录值...");

            action = "ModifyRecord";
            let mut payloads = vec![];
            for record in ipv6_record.iter().clone() {
                payloads.push(json!({
                    "Domain": domain,
                    "RecordType": record.record_type,
                    "RecordLine": record.line,
                    "Value": local_ipv6_set[0].to_string(),
                    "RecordId": record.record_id,
                    "SubDomain": record.name
                }
                ));
            }

            for (index, payload) in payloads.iter().enumerate() {
                log::warn!(
                    "RecordId: {},原记录值:{}",
                    ipv6_record[index].record_id,
                    ipv6_record[index].value
                );
                match client.request(action, version, payload.clone()).await {
                    Ok(response) => {
                        if response.contains("Error") {
                            log::error!("修改失败: {response}");
                        } else {
                            log::warn!("成功修改记录值为: {}", local_ipv6_set[0]);
                        }
                    }
                    Err(e) => {
                        log::error!("Error: {}", e);
                    }
                }
            }
        }

        interval.tick().await;
    }
}

async fn get_ipv6_set_with_iface(iface: &str) -> Result<Vec<Ipv6Addr>, Box<dyn std::error::Error>> {
    let if_inet6 = tokio::fs::read_to_string("/proc/net/if_inet6")
        .await
        .map_err(|e| format!("读取ipv6失败: {}", e))?;

    let ipv6_set: Vec<Ipv6Addr> = if_inet6
        .split('\n')
        .filter_map(|ipv6| {
            if ipv6.contains(iface) && !ipv6.contains("fe80") {
                let (value, _) = ipv6.split_at(32);
                let value_format = format!(
                    "{}:{}:{}:{}:{}:{}:{}:{}",
                    &value[0..4],
                    &value[4..8],
                    &value[8..12],
                    &value[12..16],
                    &value[16..20],
                    &value[20..24],
                    &value[24..28],
                    &value[28..32]
                );
                Some(Ipv6Addr::from_str(&value_format).unwrap())
            } else {
                None
            }
        })
        .collect();

    Ok(ipv6_set)
}
