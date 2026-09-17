pub fn backend_port_from_env(raw: Option<&str>) -> u16 {
    raw.and_then(|v| v.parse().ok())
        .filter(|&p| p > 0)
        .unwrap_or(4545)
}

pub fn request_max_in_flight_from_env(raw: Option<&str>) -> usize {
    raw.and_then(|v| v.parse().ok()).unwrap_or(100)
}

pub fn mysql_startup_connect_attempts_from_env(raw: Option<&str>) -> u32 {
    raw.and_then(|v| v.parse().ok())
        .filter(|&n| (1..=30).contains(&n))
        .unwrap_or(10)
}

pub fn mysql_acquire_timeout_secs_from_env(raw: Option<&str>) -> u64 {
    raw.and_then(|v| v.parse().ok())
        .filter(|&n| (1..=120).contains(&n))
        .unwrap_or(10)
}

pub struct MysqlPoolOpts {
    pub test_before_acquire: bool,
    pub max_connections: u32,
}

pub fn mysql_pool_options_from_env_values(
    test_before: Option<&str>,
    max_conn: Option<&str>,
) -> MysqlPoolOpts {
    let test_before_acquire = match test_before.map(str::trim).map(str::to_ascii_lowercase) {
        Some(v) if matches!(v.as_str(), "0" | "false" | "no" | "off") => false,
        _ => true,
    };
    let max_connections = max_conn
        .and_then(|v| v.parse().ok())
        .filter(|&n| (1..=200).contains(&n))
        .unwrap_or(20);
    MysqlPoolOpts {
        test_before_acquire,
        max_connections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port_is_4545() {
        assert_eq!(backend_port_from_env(None), 4545);
        assert_eq!(backend_port_from_env(Some("4546")), 4546);
    }
}
