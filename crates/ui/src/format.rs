use app_contracts::features::activity::Clock;
use app_contracts::features::settings::{ByteUnits, NetworkUnits, Units};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rate {
    Bytes(ByteUnits),
    Bits,
}

impl Rate {
    pub fn network(units: Units) -> Self {
        match units.network {
            NetworkUnits::Bits => Self::Bits,
            NetworkUnits::Bytes => Self::Bytes(units.bytes),
        }
    }

    pub fn disk(units: Units) -> Self {
        Self::Bytes(units.bytes)
    }
}

pub fn rate(rate: Rate, bytes: u64) -> String {
    match rate {
        Rate::Bytes(units) => bytes_per_second(units, bytes),
        Rate::Bits => bits_per_second(bytes.saturating_mul(8)),
    }
}

struct Decimal;

#[expect(non_upper_case_globals)]
impl Decimal {
    const Kilo: f64 = 1000.0;
}

fn bits_per_second(bits: u64) -> String {
    let f = bits as f64;
    if f >= Decimal::Kilo.powi(3) {
        format!("{:.1} Gbps", f / Decimal::Kilo.powi(3))
    } else if f >= Decimal::Kilo.powi(2) {
        format!("{:.1} Mbps", f / Decimal::Kilo.powi(2))
    } else if f >= Decimal::Kilo {
        format!("{:.0} Kbps", f / Decimal::Kilo)
    } else {
        format!("{bits} bps")
    }
}

fn bits_bound(bits: u64) -> String {
    let f = bits as f64;
    let (value, suffix) = if f >= Decimal::Kilo.powi(3) {
        (f / Decimal::Kilo.powi(3), "Gbps")
    } else if f >= Decimal::Kilo.powi(2) {
        (f / Decimal::Kilo.powi(2), "Mbps")
    } else {
        (f / Decimal::Kilo, "Kbps")
    };
    format!("{value:.0} {suffix}")
}

pub fn percent(value: f32) -> String {
    format!("{value:.1}%")
}

pub fn ghz(mhz: u64) -> String {
    format!("{:.1}", mhz as f64 / 1000.0)
}

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

pub fn rate_bound(rate: Rate, v: u64) -> String {
    let units = match rate {
        Rate::Bytes(units) => units,
        Rate::Bits => return bits_bound(v.saturating_mul(8)),
    };
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

pub fn clock(at: Clock) -> String {
    format!("{:02}:{:02}:{:02}", at.hour, at.minute, at.second)
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
    fn a_clock_reads_in_two_digits_each() {
        let at = Clock {
            hour: 9,
            minute: 5,
            second: 0,
        };
        assert_eq!(clock(at), "09:05:00");
    }

    #[test]
    fn megahertz_read_as_gigahertz() {
        assert_eq!(ghz(3_701), "3.7");
        assert_eq!(ghz(0), "0.0");
    }

    #[test]
    fn a_percent_keeps_one_decimal() {
        assert_eq!(percent(0.0), "0.0%");
        assert_eq!(percent(12.345), "12.3%");
    }

    #[test]
    fn a_rate_bound_is_written_whole() {
        assert_eq!(
            rate_bound(Rate::Bytes(ByteUnits::Windows), 100 << 20),
            "100 MB/s"
        );
        assert_eq!(
            rate_bound(Rate::Bytes(ByteUnits::Iec), 500 << 10),
            "500 KiB/s"
        );
        assert_eq!(
            rate_bound(Rate::Bytes(ByteUnits::Windows), 1 << 30),
            "1 GB/s"
        );
        assert_eq!(rate_bound(Rate::Bits, 12_500_000), "100 Mbps");
    }

    #[test]
    fn network_in_bits_counts_in_thousands() {
        assert_eq!(rate(Rate::Bits, 0), "0 bps");
        assert_eq!(rate(Rate::Bits, 100), "800 bps");
        assert_eq!(rate(Rate::Bits, 2_500), "20 Kbps");
        assert_eq!(rate(Rate::Bits, 1_250_000), "10.0 Mbps");
        assert_eq!(rate(Rate::Bits, 125_000_000), "1.0 Gbps");
        assert_eq!(rate(Rate::Bytes(ByteUnits::Windows), 20 << 10), "20 KB/s");
    }

    #[test]
    fn disk_stays_in_bytes_whatever_network_is_in() {
        let units = Units {
            bytes: ByteUnits::Iec,
            network: NetworkUnits::Bits,
        };
        assert_eq!(Rate::disk(units), Rate::Bytes(ByteUnits::Iec));
        assert_eq!(Rate::network(units), Rate::Bits);
        assert_eq!(
            Rate::network(Units {
                network: NetworkUnits::Bytes,
                ..units
            }),
            Rate::Bytes(ByteUnits::Iec)
        );
    }
}
