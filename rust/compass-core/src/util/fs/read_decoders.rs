pub fn default<T>(_idx: usize, row: String) -> Result<T, std::io::Error>
where
    T: std::str::FromStr<Err = String>,
{
    row.parse::<T>().map_err(|e| {
        let msg = format!("failure decoding row {} due to: {:}", row, e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    })
}
