pub fn bytes_per_second(v: u64) -> String {
    format!("{}/s", bytes(v))
}

struct Binary;

#[expect(non_upper_case_globals)]
impl Binary {
    const Kib: f64 = 1024.0;
}

pub fn bytes(v: u64) -> String {
    let f = v as f64;
    if f >= Binary::Kib.powi(3) {
        format!("{:.1} GiB", f / Binary::Kib.powi(3))
    } else if f >= Binary::Kib.powi(2) {
        format!("{:.1} MiB", f / Binary::Kib.powi(2))
    } else if f >= Binary::Kib {
        format!("{:.0} KiB", f / Binary::Kib)
    } else {
        format!("{v} B")
    }
}
