/// default read decoder for an arbitrary type if the read operation
/// can be performed via the FromStr trait for type T
pub fn default<T>(_idx: usize, row: String) -> Result<T, std::io::Error>
where
    T: std::str::FromStr<Err = String>,
{
    row.parse::<T>().map_err(|e| {
        let msg = format!("failure decoding row {} due to: {:}", row, e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    })
}

pub fn string(_idx: usize, row: String) -> Result<String, std::io::Error> {
    Ok(row)
}
