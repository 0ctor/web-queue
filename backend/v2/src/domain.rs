pub fn format_display_name(full_name: &str, mode: &str) -> String {
    let parts: Vec<&str> = full_name.split_whitespace().filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return "Paciente".to_string();
    }
    match mode {
        "full" => parts.join(" "),
        "initials" => {
            let first = parts[0].chars().next().unwrap_or('?').to_uppercase().to_string();
            if parts.len() == 1 {
                return format!("{first}.");
            }
            let last = parts[parts.len() - 1]
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            format!("{first}. {last}.")
        }
        _ => {
            let first = parts[0];
            if parts.len() == 1 {
                return first.to_string();
            }
            let last_initial = parts[parts.len() - 1]
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            format!("{first} {last_initial}.")
        }
    }
}

const CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn normalize_display_code(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .take(6)
        .collect()
}

pub fn generate_display_code(random_bytes: &[u8; 6]) -> String {
    random_bytes
        .iter()
        .map(|b| CODE_CHARS[(*b as usize) % CODE_CHARS.len()] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_default_first_last_initial() {
        assert_eq!(format_display_name("Maria Silva", "first_last_initial"), "Maria S.");
    }

    #[test]
    fn display_name_full_and_initials() {
        assert_eq!(format_display_name("Maria Silva", "full"), "Maria Silva");
        assert_eq!(format_display_name("Maria Silva", "initials"), "M. S.");
    }

    #[test]
    fn display_name_empty_is_safe() {
        assert_eq!(format_display_name("   ", "first_last_initial"), "Paciente");
    }

    #[test]
    fn normalize_code_uppercases_and_trims() {
        assert_eq!(normalize_display_code("ab-12c"), "AB12C");
    }
}
