use std::io;

use tokio::io::{AsyncBufRead, AsyncBufReadExt as _, AsyncReadExt as _};

pub async fn next_line_limited<R: AsyncBufRead + Unpin>(
    read: &mut R,
    limit: usize,
) -> io::Result<Option<String>> {
    let mut buf = Vec::new();
    let n = read.take(limit as u64).read_until(b'\n', &mut buf).await?;
    if n == 0 {
        return Ok(None);
    }

    if buf.ends_with(b"\r\n") {
        buf.pop();
        buf.pop();
    } else if buf.ends_with(b"\n") {
        buf.pop();
    } else if n == limit {
        return Err(io::Error::other("line too long"));
    }

    Ok(Some(String::from_utf8(buf).map_err(|err| {
        io::Error::new(io::ErrorKind::InvalidData, err)
    })?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_next_line_limited() {
        async fn all_lines(mut read: &[u8], limit: usize) -> io::Result<Vec<String>> {
            let mut lines = Vec::new();
            while let Some(line) = next_line_limited(&mut read, limit).await? {
                lines.push(line);
            }
            Ok(lines)
        }

        assert_eq!(all_lines(b"a\r\nb\n", 16).await.unwrap(), ["a", "b"]);

        // The terminator counts towards the limit
        assert_eq!(all_lines(b"xxx\n", 4).await.unwrap(), ["xxx"]);
        assert!(all_lines(b"xxxx\n", 4).await.is_err());

        // Final line without terminator
        assert_eq!(all_lines(b"a\nb", 16).await.unwrap(), ["a", "b"]);
    }
}
