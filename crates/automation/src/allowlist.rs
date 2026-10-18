//! Command allowlist for the MCP server (`--commands <file>`).

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
enum AllowPattern {
    Exact(String),
    Prefix(String),
}

/// A parsed allowlist of command ids and prefix globs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandAllowlist {
    patterns: Vec<AllowPattern>,
}

impl CommandAllowlist {
    /// Parse an allowlist from text.
    ///
    /// File format: one command id or trailing `*` prefix glob (`layer.*`) per line.
    /// Blank lines and `#` comments are ignored. Surrounding whitespace is trimmed.
    /// Only a trailing `*` glob is supported; anything else containing `*` is a parse
    /// error naming the line. An empty rule list is rejected.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut patterns = Vec::new();
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.contains('*') {
                let Some(prefix) = line.strip_suffix('*') else {
                    return Err(format!("invalid pattern `{line}`: only a trailing `*` glob is supported"));
                };
                if prefix.contains('*') {
                    return Err(format!("invalid pattern `{line}`: only a trailing `*` glob is supported"));
                }
                patterns.push(AllowPattern::Prefix(prefix.to_string()));
            } else {
                patterns.push(AllowPattern::Exact(line.to_string()));
            }
        }
        if patterns.is_empty() {
            return Err("commands allowlist is empty".to_string());
        }
        Ok(Self { patterns })
    }

    /// Load and parse an allowlist from a file path.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Check if a command id is permitted by this allowlist.
    pub fn allows(&self, command_id: &str) -> bool {
        self.patterns.iter().any(|p| match p {
            AllowPattern::Exact(exact) => exact == command_id,
            AllowPattern::Prefix(prefix) => command_id.starts_with(prefix),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exact_and_prefix_rules() {
        let text = "
            # Allowed commands
            file.new
            layer.*
        ";
        let al = CommandAllowlist::parse(text).unwrap();
        assert!(al.allows("file.new"));
        assert!(!al.allows("file.open"));
        assert!(al.allows("layer.new.layer"));
        assert!(al.allows("layer.duplicate"));
        assert!(!al.allows("layers.x"));
    }

    #[test]
    fn parse_rejects_empty() {
        assert!(CommandAllowlist::parse("").is_err());
        assert!(CommandAllowlist::parse("   \n# only comments\n   ").is_err());
    }

    #[test]
    fn parse_rejects_malformed_globs() {
        for bad in ["*foo", "layer.*.blur", "foo*bar", "**", "layer.*.*"] {
            let err = CommandAllowlist::parse(bad).unwrap_err();
            assert!(err.contains(bad), "{bad}: {err}");
            assert!(err.contains("only a trailing `*` glob is supported"), "{bad}: {err}");
        }
    }
}
