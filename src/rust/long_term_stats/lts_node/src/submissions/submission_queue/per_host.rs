use influxdb2::{Client, models::DataPoint};
use lqos_bus::long_term_stats::StatsHost;
use pgdb::OrganizationDetails;
use futures::prelude::*;

pub async fn collect_per_host(
    org: &OrganizationDetails,
    node_id: &str,
    timestamp: i64,
    totals: &Option<Vec<StatsHost>>,
) -> anyhow::Result<()> {
    if let Some(hosts) = totals {
        let influx_url = format!("http://{}:8086", org.influx_host);
        let client = Client::new(&influx_url, &org.influx_org, &org.influx_token);
        let mut points: Vec<DataPoint> = Vec::new();
        
        for host in hosts {
            points.push(DataPoint::builder("host_bits")
                .tag("host_id", node_id.to_string())
                .tag("organization_id", org.key.to_string())
                .tag("direction", "down".to_string())
                .tag("circuit_id", host.circuit_id.to_string())
                .tag("ip", host.ip_address.to_string())
                .timestamp(timestamp)
                .field("min", host.bits.min.0 as i64)
                .field("max", host.bits.max.0 as i64)
                .field("avg", host.bits.avg.0 as i64)
                .build()?);
            points.push(DataPoint::builder("host_bits")
                .tag("host_id", node_id.to_string())
                .tag("organization_id", org.key.to_string())
                .tag("direction", "up".to_string())
                .tag("circuit_id", host.circuit_id.to_string())
                .tag("ip", host.ip_address.to_string())
                .timestamp(timestamp)
                .field("min", host.bits.min.1 as i64)
                .field("max", host.bits.max.1 as i64)
                .field("avg", host.bits.avg.1 as i64)
                .build()?);
            points.push(DataPoint::builder("rtt")
                .tag("host_id", node_id.to_string())
                .tag("organization_id", org.key.to_string())
                .tag("circuit_id", host.circuit_id.to_string())
                .tag("ip", host.ip_address.to_string())
                .timestamp(timestamp)
                .field("min", host.rtt.avg as f64 / 100.0)
                .field("max", host.rtt.max as f64 / 100.0)
                .field("avg", host.rtt.avg as f64 / 100.0)
                .build()?);
        }

        client
            .write_with_precision(
                &org.influx_bucket,
                stream::iter(points),
                influxdb2::api::write::TimestampPrecision::Seconds,
            )
            .await?;
    }
    Ok(())
}