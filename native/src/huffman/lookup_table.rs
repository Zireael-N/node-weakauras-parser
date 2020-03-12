#[derive(Clone)]
pub(crate) struct TableEntry {
    pub(crate) code_length: u8,
    pub(crate) data: TableData,
}

#[derive(Clone)]
pub(crate) enum TableData {
    Symbol(u8),
    Reference(Vec<TableEntry>),
}

// https://doc.rust-lang.org/core/slice/struct.Iter.html#method.is_sorted_by
// is Nightly-only as of Rust 1.41.1.
use std::cmp::Ordering;
#[cfg_attr(not(debug_assertions), allow(dead_code))]
fn is_sorted<T, F>(slice: &[T], comparator: F) -> bool
where
    F: Fn(&T, &T) -> Ordering,
{
    slice.windows(2).all(|w| comparator(&w[0], &w[1]) != Ordering::Greater)
}

pub(crate) fn build_lookup_table(codes: &[(u32, u8, u8)]) -> Result<Vec<TableEntry>, &'static str> {
    debug_assert!(is_sorted(codes, |prev, curr| {
        prev.1.cmp(&curr.1).then_with(|| prev.0.cmp(&curr.0))
    }));

    let mut lut = vec![
        TableEntry {
            code_length: 0,
            data: TableData::Symbol(0),
        };
        256
    ];

    for &(mut code, mut code_len, symbol) in codes.iter() {
        let mut cursor = &mut lut;
        while code_len > 8 {
            let entry = &mut cursor[(code as u8) as usize];
            if entry.code_length == 0 {
                entry.code_length = 8;
                entry.data = TableData::Reference(vec![
                    TableEntry {
                        code_length: 0,
                        data: TableData::Symbol(0),
                    };
                    256
                ]);
            }

            if let TableData::Reference(ref mut v) = entry.data {
                cursor = v;
            } else {
                return Err("compression error"); // two values have the same prefix
            }
            code >>= 8;
            code_len -= 8;
        }
        if code_len < 8 {
            for prefix in 0..(1 << (8 - code_len)) {
                let entry = &mut cursor[((prefix << code_len) | code as u8) as usize];
                entry.code_length = code_len;
                entry.data = TableData::Symbol(symbol);
            }
        } else {
            let entry = &mut cursor[(code as u8) as usize];
            entry.code_length = code_len;
            entry.data = TableData::Symbol(symbol);
        }
    }

    Ok(lut)
}
