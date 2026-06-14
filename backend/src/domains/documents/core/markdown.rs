pub(super) fn extract_markdown_text(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| match markdown_heading_text(line.trim_end()) {
            Some(heading_text) => heading_text,
            None => line.trim_end(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn markdown_heading_text(line: &str) -> Option<&str> {
    let mut hash_count = 0;
    for character in line.chars() {
        if character == '#' {
            hash_count += 1;
            continue;
        }
        break;
    }

    if !(1..=6).contains(&hash_count) {
        return None;
    }

    line.as_bytes()
        .get(hash_count)
        .filter(|byte| **byte == b' ')
        .map(|_| &line[hash_count + 1..])
}
