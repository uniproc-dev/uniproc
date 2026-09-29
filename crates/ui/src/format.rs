use app_contracts::features::settings::ByteUnits;

pub fn bytes_per_second(units: ByteUnits, v: u64) -> String {
    format!("{}/s", bytes(units, v))
}

struct Binary;

#[expect(non_upper_case_globals)]
impl Binary {
    const Kib: f64 = 1024.0;
}

fn suffixes(units: ByteUnits) -> [&'static str; 3] {
    match units {
        ByteUnits::Windows => ["KB", "MB", "GB"],
        ByteUnits::Iec => ["KiB", "MiB", "GiB"],
    }
}

pub fn rate_bound(units: ByteUnits, v: u64) -> String {
    let [kilo, mega, giga] = suffixes(units);
    let f = v as f64;
    let (value, suffix) = if f >= Binary::Kib.powi(3) {
        (f / Binary::Kib.powi(3), giga)
    } else if f >= Binary::Kib.powi(2) {
        (f / Binary::Kib.powi(2), mega)
    } else {
        (f / Binary::Kib, kilo)
    };
    format!("{value:.0} {suffix}/s")
}

pub fn bytes(units: ByteUnits, v: u64) -> String {
    let [kilo, mega, giga] = suffixes(units);
    let f = v as f64;
    if f >= Binary::Kib.powi(3) {
        format!("{:.1} {giga}", f / Binary::Kib.powi(3))
    } else if f >= Binary::Kib.powi(2) {
        format!("{:.1} {mega}", f / Binary::Kib.powi(2))
    } else if f >= Binary::Kib {
        format!("{:.0} {kilo}", f / Binary::Kib)
    } else {
        format!("{v} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_units_count_in_1024s_and_say_gb() {
        assert_eq!(bytes(ByteUnits::Windows, 3 << 30), "3.0 GB");
        assert_eq!(bytes(ByteUnits::Iec, 3 << 30), "3.0 GiB");
        assert_eq!(bytes_per_second(ByteUnits::Windows, 20 << 10), "20 KB/s");
        assert_eq!(bytes(ByteUnits::Iec, 512), "512 B");
    }

    #[test]
    fn a_rate_bound_is_written_whole() {
        assert_eq!(rate_bound(ByteUnits::Windows, 100 << 20), "100 MB/s");
        assert_eq!(rate_bound(ByteUnits::Iec, 500 << 10), "500 KiB/s");
        assert_eq!(rate_bound(ByteUnits::Windows, 1 << 30), "1 GB/s");
    }
}
