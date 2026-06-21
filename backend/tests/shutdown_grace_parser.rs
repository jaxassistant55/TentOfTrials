#[cfg(test)]
mod tests {
    const ENV_SHUTDOWN_GRACE_SECS: &str = "TOT_SHUTDOWN_GRACE_SECS";

    fn parse_shutdown_grace_secs() -> u64 {
        // Same logic as main.rs
        match std::env::var(ENV_SHUTDOWN_GRACE_SECS) {
            Ok(v) => match v.parse::<u64>() {
                Ok(n) if n > 0 && n <= 300 => n,
                _ => 30,
            },
            Err(_) => 30,
        }
    }

    #[test]
    fn test_shutdown_grace_default() {
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
        assert_eq!(parse_shutdown_grace_secs(), 30);
    }

    #[test]
    fn test_shutdown_grace_valid() {
        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "60");
        assert_eq!(parse_shutdown_grace_secs(), 60);
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
    }

    #[test]
    fn test_shutdown_grace_zero() {
        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "0");
        assert_eq!(parse_shutdown_grace_secs(), 30); // invalid → default
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
    }

    #[test]
    fn test_shutdown_grace_negative() {
        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "-5");
        assert_eq!(parse_shutdown_grace_secs(), 30); // invalid → default
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
    }

    #[test]
    fn test_shutdown_grace_too_large() {
        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "999");
        assert_eq!(parse_shutdown_grace_secs(), 30); // > 300 → default
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
    }

    #[test]
    fn test_shutdown_grace_non_numeric() {
        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "abc");
        assert_eq!(parse_shutdown_grace_secs(), 30); // invalid → default
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
    }

    #[test]
    fn test_shutdown_grace_boundary_valid() {
        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "1");
        assert_eq!(parse_shutdown_grace_secs(), 1);
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);

        std::env::set_var(ENV_SHUTDOWN_GRACE_SECS, "300");
        assert_eq!(parse_shutdown_grace_secs(), 300);
        std::env::remove_var(ENV_SHUTDOWN_GRACE_SECS);
    }
}
