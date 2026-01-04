pub fn line_frame(message: &str) -> Vec<u8> {
    format!("{message}\n").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_frame_adds_newline() {
        assert_eq!(line_frame("OK"), b"OK\n");
    }

    #[test]
    fn line_frame_keeps_message_content() {
        assert_eq!(line_frame("ERR invalid command"), b"ERR invalid command\n");
    }

    #[test]
    fn line_frame_supports_empty_message() {
        assert_eq!(line_frame(""), b"\n");
    }
}
